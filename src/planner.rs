use crate::actions::Action;
use crate::conditions::{Condition, Direction};
use crate::goals::Goal;
use crate::state::{State, StateOperation, StateType, StateVar, fixed_point_from_f64, shift};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::error::Error;
use std::fmt;

/// Errors that can occur during planning.
///
/// The first two are outcomes of a search; the other four are mistakes in
/// the problem (the state, goal and actions handed to `plan`) and are
/// reported before any search runs. `PartialEq` but not `Eq`, because
/// [`PlannerError::InvalidActionCost`] carries the offending `f64`.
#[derive(Debug, Clone, PartialEq)]
pub enum PlannerError {
    /// The goal is proved unreachable: either the up-front reachability
    /// analysis rejected it or the search exhausted the reachable state space.
    NoPlanFound,
    /// The search expanded `expanded` states without finding a plan and gave
    /// up. The goal may be unreachable, or the problem may be larger than the
    /// configured budget (see [`Planner::max_expansions`]).
    SearchLimitExceeded {
        /// Number of states expanded before the search stopped.
        expanded: usize,
    },
    /// An action has a cost that is negative, NaN, infinite or above
    /// [`StateVar::MAX_F64`].
    InvalidActionCost {
        /// Name of the action.
        action: String,
        /// The cost as stored on the action.
        cost: f64,
    },
    /// Two mentions of the same variable disagree about its type, for
    /// example an `i64` in the initial state and an `at_least(5.0)` in a goal.
    IncompatibleStateTypes {
        /// Name of the variable.
        variable: String,
        /// The type the variable was first seen with. The initial state wins
        /// if it has the variable; otherwise the first mention in the goal
        /// or the actions, in order.
        expected: StateType,
        /// Where `expected` comes from.
        expected_origin: VariableOrigin,
        /// The conflicting type.
        found: StateType,
        /// Where `found` comes from.
        found_origin: VariableOrigin,
    },
    /// A condition that can never be satisfied because of its shape: an
    /// ordered comparison on a non-numeric value, or a `Between` whose
    /// bounds have different types. See [`Condition::is_well_formed`].
    InvalidCondition {
        /// Name of the variable the condition is on.
        variable: String,
        /// The goal or action the condition belongs to.
        origin: VariableOrigin,
        /// The condition itself.
        condition: Condition,
    },
    /// An `Add` or `Subtract` effect whose delta is not `I64` or `F64`.
    InvalidEffect {
        /// Name of the variable the effect is on.
        variable: String,
        /// Name of the action.
        action: String,
        /// The effect itself.
        operation: StateOperation,
    },
}

/// Where the planner saw a variable mentioned, for type errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariableOrigin {
    /// The initial state passed to `plan`.
    InitialState,
    /// The conditions of the named goal.
    Goal(String),
    /// The preconditions of the named action.
    Preconditions(String),
    /// The effects of the named action.
    Effects(String),
}

impl fmt::Display for VariableOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariableOrigin::InitialState => write!(f, "the initial state"),
            VariableOrigin::Goal(name) => write!(f, "goal '{name}'"),
            VariableOrigin::Preconditions(name) => {
                write!(f, "the preconditions of action '{name}'")
            }
            VariableOrigin::Effects(name) => write!(f, "the effects of action '{name}'"),
        }
    }
}

impl fmt::Display for PlannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlannerError::NoPlanFound => write!(f, "No plan found"),
            PlannerError::SearchLimitExceeded { expanded } => write!(
                f,
                "Search stopped after expanding {expanded} states without finding a plan. \
                 The goal may be unreachable; if the problem is just large, raise the \
                 limit with Planner::max_expansions"
            ),
            PlannerError::InvalidActionCost { action, cost } => write!(
                f,
                "Invalid action cost: action '{action}' has cost {cost}; costs must be finite, \
                 non-negative and at most {}",
                StateVar::MAX_F64
            ),
            PlannerError::IncompatibleStateTypes {
                variable,
                expected,
                expected_origin,
                found,
                found_origin,
            } => write!(
                f,
                "Incompatible state types: variable '{variable}' is {expected} in \
                 {expected_origin} but {found} in {found_origin}"
            ),
            PlannerError::InvalidCondition {
                variable,
                origin,
                condition,
            } => write!(
                f,
                "Invalid condition: '{variable} {condition}' in {origin} can never be \
                 satisfied; ordered comparisons need i64 or f64 values of one type"
            ),
            PlannerError::InvalidEffect {
                variable,
                action,
                operation,
            } => write!(
                f,
                "Invalid effect: effect on '{variable}' in action '{action}' would \
                 {operation}, but only i64 and f64 values can be added or subtracted"
            ),
        }
    }
}

impl Error for PlannerError {}

/// A plan represents a sequence of actions that will achieve a goal.
/// It includes the actions to perform and the total cost of execution.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// The sequence of actions to perform in order
    pub actions: Vec<Action>,
    /// The total cost of executing all actions in the plan. Costs are summed
    /// exactly in thousandths, so ten actions of cost `0.1` give `1.0`, not
    /// `0.9999999999999999`.
    pub cost: f64,
}

/// A cost in thousandths, the planner's internal unit. Costs enter as `f64`
/// through [`Action::cost`] and are converted once per `plan` call, so every
/// comparison and sum in the search is an exact integer operation. Never
/// negative; sums saturate instead of overflowing.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
struct Cost(u64);

impl Cost {
    const ZERO: Cost = Cost(0);
    /// Larger than any real cost; used as "no action can do this".
    const UNREACHABLE: Cost = Cost(u64::MAX);

    /// Converts an action cost. `None` for negative, NaN, infinite or
    /// out-of-range values (above [`StateVar::MAX_F64`]).
    fn try_from_f64(cost: f64) -> Option<Cost> {
        match fixed_point_from_f64(cost) {
            Some(units) if units >= 0 => Some(Cost(units as u64)),
            _ => None,
        }
    }

    fn to_f64(self) -> f64 {
        self.0 as f64 / 1000.0
    }

    fn saturating_add(self, other: Cost) -> Cost {
        Cost(self.0.saturating_add(other.0))
    }
}

/// Cheapest known price for moving a numeric variable: `cost` per `per_unit`
/// raw units, as an exact fraction. `per_unit` is never zero.
#[derive(Clone, Copy, Debug)]
struct StepRate {
    cost: Cost,
    per_unit: u64,
}

impl StepRate {
    /// `self.cost / self.per_unit < other.cost / other.per_unit`, without
    /// division. Both products fit in a `u128` because each factor is below
    /// 2^64.
    fn is_cheaper_than(&self, other: &StepRate) -> bool {
        (self.cost.0 as u128) * (other.per_unit as u128)
            < (other.cost.0 as u128) * (self.per_unit as u128)
    }

    fn cheaper(self, other: StepRate) -> StepRate {
        if other.is_cheaper_than(&self) {
            other
        } else {
            self
        }
    }

    /// A lower bound on the cost of moving `distance` raw units at this
    /// rate: `floor(distance * cost / per_unit)`, saturating. The continuous
    /// relaxation is already a lower bound on any real mix of step actions,
    /// and rounding down can only lower it further.
    ///
    /// The product fits in a `u64` for every realistic distance and cost, and
    /// a 64-bit division is a single instruction where a 128-bit one is a
    /// library call; the `u128` path only runs on overflow and gives the
    /// same floor.
    #[inline]
    fn price(&self, distance: u64) -> Cost {
        match distance.checked_mul(self.cost.0) {
            Some(product) => Cost(product / self.per_unit),
            None => {
                let exact = (distance as u128) * (self.cost.0 as u128) / (self.per_unit as u128);
                Cost(exact.min(u64::MAX as u128) as u64)
            }
        }
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Plan (total cost: {:.1}):", self.cost)?;
        for (i, action) in self.actions.iter().enumerate() {
            writeln!(f, "Step {}: {}", i + 1, action)?;
        }
        Ok(())
    }
}

/// A planner that uses A* search to find the cheapest sequence of actions
/// that satisfies a goal.
///
/// The heuristic is derived from the action set and never overestimates the
/// remaining cost, so the returned plan is guaranteed to be optimal (no other
/// sequence of the given actions reaches the goal more cheaply).
///
/// Before searching, the planner runs a relaxed reachability analysis over
/// the action set. Goals it can prove unreachable return
/// [`PlannerError::NoPlanFound`] immediately, and actions that can never fire
/// are dropped. Goals it cannot prove unreachable are searched under an
/// expansion budget ([`Planner::max_expansions`], default
/// [`DEFAULT_MAX_EXPANSIONS`]) so that an unbounded state space can never
/// make `plan` run forever.
///
/// The planner holds only configuration and can be reused for multiple
/// planning requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Planner {
    max_expansions: Option<usize>,
}

/// Default search budget: the number of states `Planner::plan` expands before
/// giving up with [`PlannerError::SearchLimitExceeded`].
pub const DEFAULT_MAX_EXPANSIONS: usize = 100_000;

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner {
    /// Creates a planner with the default budget of
    /// [`DEFAULT_MAX_EXPANSIONS`] expansions.
    pub fn new() -> Self {
        Planner {
            max_expansions: Some(DEFAULT_MAX_EXPANSIONS),
        }
    }

    /// Stops the search after `n` expansions (states popped from the open set
    /// and expanded) and returns [`PlannerError::SearchLimitExceeded`].
    ///
    /// Every shipped example finishes in under 4 000 expansions; the default
    /// of 100 000 costs under 100 ms when fully spent.
    pub fn max_expansions(mut self, n: usize) -> Self {
        self.max_expansions = Some(n);
        self
    }

    /// Removes the expansion budget. The search then runs until it finds a
    /// plan or exhausts the reachable state space. Only safe when that space
    /// is finite: if some action can increase a numeric variable without
    /// limit and the goal is unreachable, `plan` will never return.
    pub fn without_expansion_limit(mut self) -> Self {
        self.max_expansions = None;
        self
    }

    /// The configured expansion budget, or `None` when unlimited.
    pub fn expansion_limit(&self) -> Option<usize> {
        self.max_expansions
    }

    /// Finds the cheapest plan that achieves `goal` starting from
    /// `initial_state`.
    ///
    /// # Errors
    ///
    /// * [`PlannerError::InvalidActionCost`] if any action has a negative,
    ///   NaN or infinite cost, or one above [`StateVar::MAX_F64`].
    /// * [`PlannerError::IncompatibleStateTypes`] if a condition or effect
    ///   uses a different type than the variable it refers to (for example
    ///   an `i64` requirement on an `f64` variable).
    /// * [`PlannerError::InvalidCondition`] / [`PlannerError::InvalidEffect`]
    ///   if a condition or effect is malformed (an ordered comparison on a
    ///   `bool`, a `Bool` delta).
    /// * [`PlannerError::NoPlanFound`] if the goal is proved unreachable,
    ///   either by the up-front reachability analysis or by exhausting the
    ///   reachable state space.
    /// * [`PlannerError::SearchLimitExceeded`] if the expansion budget runs
    ///   out first. The goal may be unreachable in a way the analysis cannot
    ///   prove, or the problem may need a larger budget.
    pub fn plan(
        &self,
        initial_state: &State,
        goal: &Goal,
        actions: &[Action],
    ) -> Result<Plan, PlannerError> {
        let costs = validate_costs(actions)?;
        validate_types(initial_state, goal, actions)?;

        let reach = Reachability::analyse(initial_state, actions);
        if !reach.goal_may_be_reachable(goal) {
            return Err(PlannerError::NoPlanFound);
        }
        // Only actions that can possibly ever fire take part in the search.
        let live: Vec<&Action> = reach.applicable.iter().map(|&i| &actions[i]).collect();
        let live_costs: Vec<Cost> = reach.applicable.iter().map(|&i| costs[i]).collect();

        // From here on the search works on slot-indexed states (plan 007):
        // variable names are interned once and never hashed again.
        let compiled = Compiled::new(initial_state, goal, &live, &live_costs);
        let mut analysis = GoalAnalysis::new(&compiled);

        let Some(initial_h) = analysis.estimate(&compiled.initial) else {
            return Err(PlannerError::NoPlanFound);
        };

        let mut graph = SearchGraph::new(&compiled.initial, initial_h);
        let mut open = BinaryHeap::new();
        open.push(OpenEntry {
            f: initial_h,
            g: Cost::ZERO,
            id: 0,
        });

        let mut expanded = 0usize;
        while let Some(OpenEntry { g, id, .. }) = open.pop() {
            // A cheaper path to this node was found after this entry was
            // pushed; the newer entry will handle it.
            if g > graph.nodes[id].g {
                continue;
            }

            if is_met(&compiled.goal, graph.state(id)) {
                return Ok(reconstruct_path(&graph.nodes, &live, &reach.applicable, id));
            }

            if let Some(limit) = self.max_expansions
                && expanded >= limit
            {
                return Err(PlannerError::SearchLimitExceeded { expanded });
            }
            expanded += 1;

            for (action_idx, action) in compiled.actions.iter().enumerate() {
                if !action.can_execute(graph.state(id)) {
                    continue;
                }
                // The child is built in place at the end of the arena and
                // only kept if it turns out to be new.
                action.apply(graph.begin_child(id));
                let tentative_g = g.saturating_add(action.cost);

                let (hash, existing) = graph.find_child();
                if let Some(next_id) = existing
                    && tentative_g >= graph.nodes[next_id].g
                {
                    graph.discard_child();
                    continue;
                }

                // `h` is a function of the state, so a known node keeps its
                // estimate instead of recomputing it.
                let h = match existing {
                    Some(next_id) => graph.nodes[next_id].h,
                    None => match analysis.estimate(graph.child()) {
                        Some(h) => h,
                        None => {
                            graph.discard_child();
                            continue;
                        }
                    },
                };

                let next_id = match existing {
                    Some(next_id) => {
                        graph.nodes[next_id].g = tentative_g;
                        graph.nodes[next_id].parent = Some((id, action_idx));
                        graph.discard_child();
                        next_id
                    }
                    None => graph.commit_child(hash, tentative_g, h, Some((id, action_idx))),
                };

                open.push(OpenEntry {
                    f: tentative_g.saturating_add(h),
                    g: tentative_g,
                    id: next_id,
                });
            }
        }

        Err(PlannerError::NoPlanFound)
    }

    /// Plans for the highest-priority goal that has a plan.
    ///
    /// Goals are tried in descending [`Goal::priority`]; equal priorities
    /// keep their order in `goals`. Returns the index of the first goal with
    /// a plan, and the plan. This is the goal-selection loop of a GOAP agent:
    ///
    /// ```
    /// use goap::prelude::*;
    /// let state = State::builder().set("hungry", true).set("gold", 0).build();
    /// let goals = [
    ///     Goal::builder("get_rich").priority(5).requires("gold", at_least(100)).build(),
    ///     Goal::builder("eat").priority(10).requires("hungry", false).build(),
    /// ];
    /// let actions = [Action::builder("eat").sets("hungry", false).build()];
    /// let (index, plan) = Planner::new().plan_by_priority(&state, &goals, &actions).unwrap();
    /// assert_eq!(goals[index].name, "eat");
    /// assert_eq!(plan.actions.len(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// * [`PlannerError::InvalidActionCost`], [`PlannerError::IncompatibleStateTypes`],
    ///   [`PlannerError::InvalidCondition`] and [`PlannerError::InvalidEffect`]
    ///   abort immediately: they are mistakes in the problem, not properties
    ///   of one goal.
    /// * [`PlannerError::NoPlanFound`] and [`PlannerError::SearchLimitExceeded`]
    ///   move on to the next goal.
    /// * If no goal has a plan: `SearchLimitExceeded` from the
    ///   highest-priority goal whose search ran out of budget (it may be
    ///   achievable with a larger budget), otherwise `NoPlanFound` (every
    ///   goal was proved impossible, or `goals` is empty).
    pub fn plan_by_priority(
        &self,
        initial_state: &State,
        goals: &[Goal],
        actions: &[Action],
    ) -> Result<(usize, Plan), PlannerError> {
        let mut order: Vec<usize> = (0..goals.len()).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(goals[i].priority));

        let mut budget_hit: Option<PlannerError> = None;
        for i in order {
            match self.plan(initial_state, &goals[i], actions) {
                Ok(plan) => return Ok((i, plan)),
                Err(PlannerError::NoPlanFound) => {}
                Err(e @ PlannerError::SearchLimitExceeded { .. }) => {
                    budget_hit.get_or_insert(e);
                }
                Err(e) => return Err(e),
            }
        }
        Err(budget_hit.unwrap_or(PlannerError::NoPlanFound))
    }
}

/// Converts every action's cost to thousandths, rejecting negative, NaN,
/// infinite and out-of-range costs. A negative cost would make the search
/// loop forever; the others have no fixed-point representation. The result
/// is aligned with `actions`.
fn validate_costs(actions: &[Action]) -> Result<Vec<Cost>, PlannerError> {
    actions
        .iter()
        .map(|action| {
            Cost::try_from_f64(action.cost).ok_or_else(|| PlannerError::InvalidActionCost {
                action: action.name.clone(),
                cost: action.cost,
            })
        })
        .collect()
}

/// Checks that every condition and effect is well formed and that all
/// mentions of a variable agree on its type. The initial state is the
/// reference where it has the variable; otherwise the first mention wins.
/// This lets the search assume type consistency and reports user mistakes
/// (such as an `i64` requirement on an `f64` variable) before any work is
/// done.
fn validate_types<'a>(
    initial_state: &'a State,
    goal: &'a Goal,
    actions: &'a [Action],
) -> Result<(), PlannerError> {
    let mut types = TypeCheck {
        known: initial_state
            .iter()
            .map(|(key, value)| {
                (
                    key.as_str(),
                    (value.state_type(), VariableOrigin::InitialState),
                )
            })
            .collect(),
    };

    let origin = VariableOrigin::Goal(goal.name.clone());
    for (key, cond) in goal.conditions.iter() {
        types.check_condition(key, cond, &origin)?;
    }
    for action in actions {
        let origin = VariableOrigin::Preconditions(action.name.clone());
        for (key, cond) in action.preconditions.iter() {
            types.check_condition(key, cond, &origin)?;
        }
        let origin = VariableOrigin::Effects(action.name.clone());
        for (key, op) in &action.effects {
            if !matches!(op, StateOperation::Set(_)) && op.signed_delta().is_none() {
                return Err(PlannerError::InvalidEffect {
                    variable: key.clone(),
                    action: action.name.clone(),
                    operation: op.clone(),
                });
            }
            types.check(key, op.state_type(), &origin)?;
        }
    }
    Ok(())
}

/// The type each variable was first seen with, and where. Keys borrow from
/// the state, goal and actions being validated.
struct TypeCheck<'a> {
    known: HashMap<&'a str, (StateType, VariableOrigin)>,
}

impl<'a> TypeCheck<'a> {
    fn check(
        &mut self,
        key: &'a str,
        found: StateType,
        origin: &VariableOrigin,
    ) -> Result<(), PlannerError> {
        match self.known.get(key) {
            Some((expected, expected_origin)) if *expected != found => {
                Err(PlannerError::IncompatibleStateTypes {
                    variable: key.to_string(),
                    expected: *expected,
                    expected_origin: expected_origin.clone(),
                    found,
                    found_origin: origin.clone(),
                })
            }
            Some(_) => Ok(()),
            None => {
                self.known.insert(key, (found, origin.clone()));
                Ok(())
            }
        }
    }

    fn check_condition(
        &mut self,
        key: &'a str,
        cond: &Condition,
        origin: &VariableOrigin,
    ) -> Result<(), PlannerError> {
        if !cond.is_well_formed() {
            return Err(PlannerError::InvalidCondition {
                variable: key.to_string(),
                origin: origin.clone(),
                condition: cond.clone(),
            });
        }
        self.check(key, cond.state_type(), origin)
    }
}

/// A discovered state in the search graph. The state itself lives in
/// [`SearchGraph::arena`].
struct Node {
    /// Cheapest known cost from the initial state to this node.
    g: Cost,
    /// Heuristic estimate from this node; a function of the state alone.
    h: Cost,
    /// `hash_state` of the node's state, kept so that growing the table
    /// never rehashes a state.
    hash: u64,
    /// `(parent node, index of the action that led here)`.
    parent: Option<(usize, usize)>,
}

/// Every discovered state, flat in one arena, with an open-addressing hash
/// table over them for the duplicate check (plan 007, D4).
///
/// Node `id` owns `arena[id * width .. (id + 1) * width]`. A child is built
/// at the tail of the arena ([`SearchGraph::begin_child`]) and either kept
/// ([`SearchGraph::commit_child`]) or truncated away
/// ([`SearchGraph::discard_child`]); neither path allocates.
struct SearchGraph {
    width: usize,
    arena: Vec<Value>,
    nodes: Vec<Node>,
    /// Linear probing over node ids, capacity a power of two, load factor
    /// at most one half. Nodes are never removed, so there are no tombstones.
    table: Vec<usize>,
}

/// Empty slot marker in [`SearchGraph::table`].
const EMPTY: usize = usize::MAX;

impl SearchGraph {
    fn new(initial: &[Value], h: Cost) -> Self {
        let hash = hash_state(initial);
        let mut graph = SearchGraph {
            width: initial.len(),
            arena: initial.to_vec(),
            nodes: vec![Node {
                g: Cost::ZERO,
                h,
                hash,
                parent: None,
            }],
            table: vec![EMPTY; 16],
        };
        graph.insert_into_table(0);
        graph
    }

    /// The state of node `id`.
    fn state(&self, id: usize) -> &[Value] {
        &self.arena[id * self.width..(id + 1) * self.width]
    }

    /// Copies node `parent`'s state to the tail of the arena and returns it
    /// for the effects to edit. Must be followed by `commit_child` or
    /// `discard_child`.
    fn begin_child(&mut self, parent: usize) -> &mut [Value] {
        debug_assert_eq!(self.arena.len(), self.nodes.len() * self.width);
        self.arena
            .extend_from_within(parent * self.width..(parent + 1) * self.width);
        let start = self.arena.len() - self.width;
        &mut self.arena[start..]
    }

    /// The child being built.
    fn child(&self) -> &[Value] {
        &self.arena[self.arena.len() - self.width..]
    }

    /// Hashes the child being built and looks it up; returns the hash and
    /// the id of an identical known node, if any.
    fn find_child(&self) -> (u64, Option<usize>) {
        let child = self.child();
        let hash = hash_state(child);
        let mask = self.table.len() - 1;
        let mut i = hash as usize & mask;
        loop {
            let id = self.table[i];
            if id == EMPTY {
                return (hash, None);
            }
            if self.nodes[id].hash == hash && self.state(id) == child {
                return (hash, Some(id));
            }
            i = (i + 1) & mask;
        }
    }

    fn discard_child(&mut self) {
        let len = self.arena.len() - self.width;
        self.arena.truncate(len);
    }

    /// Keeps the child as a new node and returns its id.
    fn commit_child(
        &mut self,
        hash: u64,
        g: Cost,
        h: Cost,
        parent: Option<(usize, usize)>,
    ) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node { g, h, hash, parent });
        if self.nodes.len() * 2 > self.table.len() {
            self.grow_table();
        }
        self.insert_into_table(id);
        id
    }

    fn insert_into_table(&mut self, id: usize) {
        let mask = self.table.len() - 1;
        let mut i = self.nodes[id].hash as usize & mask;
        while self.table[i] != EMPTY {
            i = (i + 1) & mask;
        }
        self.table[i] = id;
    }

    fn grow_table(&mut self) {
        self.table = vec![EMPTY; self.table.len() * 2];
        for id in 0..self.nodes.len() {
            self.insert_into_table(id);
        }
    }
}

/// Hash of a search state: an Fx-style multiply-rotate over the tag and
/// payload of every value, with a final avalanche so that the low bits used
/// for table indexing depend on every input bit. Not cryptographic; the
/// inputs are the program's own problem.
fn hash_state(values: &[Value]) -> u64 {
    const K: u64 = 0x517c_c1b7_2722_0a95;
    let mut hash: u64 = 0;
    for value in values {
        let (tag, payload) = value.hash_words();
        hash = (hash.rotate_left(5) ^ tag).wrapping_mul(K);
        hash = (hash.rotate_left(5) ^ payload).wrapping_mul(K);
    }
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    hash ^ (hash >> 32)
}

/// Entry in the A* priority queue. Ordered so that the smallest `f` pops
/// first; ties prefer the larger `g` (the deeper node), which reaches goals
/// sooner without affecting optimality.
#[derive(Clone, Copy, Debug)]
struct OpenEntry {
    f: Cost,
    g: Cost,
    id: usize,
}

impl PartialEq for OpenEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for OpenEntry {}

impl Ord for OpenEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then_with(|| self.g.cmp(&other.g))
    }
}

impl PartialOrd for OpenEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Builds the plan by walking parent links back from the goal node.
/// `live` are the actions the search ran over; `live_to_caller` maps them
/// back to positions in the caller's slice (kept for debugging and for the
/// invariant that the plan's actions are the caller's actions).
fn reconstruct_path(
    nodes: &[Node],
    live: &[&Action],
    live_to_caller: &[usize],
    goal_id: usize,
) -> Plan {
    debug_assert_eq!(live.len(), live_to_caller.len());
    let mut indices = Vec::new();
    let mut current = goal_id;
    while let Some((parent, action_idx)) = nodes[current].parent {
        indices.push(action_idx);
        current = parent;
    }
    indices.reverse();

    let plan_actions: Vec<Action> = indices.iter().map(|&i| live[i].clone()).collect();
    Plan {
        actions: plan_actions,
        // The goal node's `g` is the exact integer sum along the path.
        cost: nodes[goal_id].g.to_f64(),
    }
}

// ---------------------------------------------------------------------------
// Compiled problem (plan 007)
// ---------------------------------------------------------------------------

/// A variable's value inside the search: [`StateVar`] with strings replaced
/// by their index in [`Compiled::strings`] and an explicit `Absent` for a
/// variable the state does not have. `Copy`, 16 bytes, so copying a state is
/// a `memcpy` and comparing two is a loop of word compares.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Value {
    Absent,
    Bool(bool),
    I64(i64),
    F64(i64),
    Str(u32),
}

impl Value {
    fn compile<'a>(var: &'a StateVar, strings: &mut Interner<'a>) -> Value {
        match var {
            StateVar::Bool(b) => Value::Bool(*b),
            StateVar::I64(n) => Value::I64(*n),
            StateVar::F64(n) => Value::F64(*n),
            StateVar::String(s) => Value::Str(strings.intern(s) as u32),
        }
    }

    /// Mirror of `StateVar::numeric_zero`.
    fn numeric_zero(self) -> Option<Value> {
        match self {
            Value::I64(_) => Some(Value::I64(0)),
            Value::F64(_) => Some(Value::F64(0)),
            _ => None,
        }
    }

    /// Raw payload of a numeric value with a tag for its kind, so that `I64`
    /// and `F64` are never compared with each other (mirror of
    /// `conditions::numeric`).
    fn numeric(self) -> Option<(bool, i64)> {
        match self {
            Value::I64(n) => Some((false, n)),
            Value::F64(n) => Some((true, n)),
            _ => None,
        }
    }

    /// Mirror of `StateVar::state_type` comparisons: same variant.
    #[inline]
    fn same_type(self, other: Value) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }

    /// `(tag, payload)` for `hash_state`.
    fn hash_words(self) -> (u64, u64) {
        match self {
            Value::Absent => (0, 0),
            Value::Bool(b) => (1, u64::from(b)),
            Value::I64(n) => (2, n as u64),
            Value::F64(n) => (3, n as u64),
            Value::Str(id) => (4, u64::from(id)),
        }
    }
}

/// Mirror of [`conditions::numeric_pair`](crate::conditions): the raw
/// values of `current` and `target` if both are numeric of the same kind.
#[inline]
fn numeric_pair(current: Value, target: Value) -> Option<(i64, i64)> {
    match (current.numeric(), target.numeric()) {
        (Some((a_kind, a)), Some((b_kind, b))) if a_kind == b_kind => Some((a, b)),
        _ => None,
    }
}

/// A [`Condition`] compiled over [`Value`]s: the same seven variants with
/// the same `is_met_by` / `distance` / `direction` semantics, including the
/// type check (which validation makes redundant inside a search, but keeps
/// the two definitions identical). `Absent` fails every test, as a missing
/// variable fails `Conditions::is_met_by`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Test {
    Equals(Value),
    NotEquals(Value),
    AtLeast(Value),
    AtMost(Value),
    MoreThan(Value),
    LessThan(Value),
    Between(Value, Value),
}

impl Test {
    fn compile<'a>(condition: &'a Condition, strings: &mut Interner<'a>) -> Test {
        let mut value = |var: &'a StateVar| Value::compile(var, strings);
        match condition {
            Condition::Equals(v) => Test::Equals(value(v)),
            Condition::NotEquals(v) => Test::NotEquals(value(v)),
            Condition::AtLeast(v) => Test::AtLeast(value(v)),
            Condition::AtMost(v) => Test::AtMost(value(v)),
            Condition::MoreThan(v) => Test::MoreThan(value(v)),
            Condition::LessThan(v) => Test::LessThan(value(v)),
            Condition::Between(lo, hi) => Test::Between(value(lo), value(hi)),
        }
    }

    /// A rank per variant, for grouping facts by kind.
    fn kind(self) -> u8 {
        match self {
            Test::Equals(_) => 0,
            Test::NotEquals(_) => 1,
            Test::AtLeast(_) => 2,
            Test::AtMost(_) => 3,
            Test::MoreThan(_) => 4,
            Test::LessThan(_) => 5,
            Test::Between(_, _) => 6,
        }
    }

    /// Mirror of `Condition::value`.
    fn value(self) -> Value {
        match self {
            Test::Equals(v)
            | Test::NotEquals(v)
            | Test::AtLeast(v)
            | Test::AtMost(v)
            | Test::MoreThan(v)
            | Test::LessThan(v)
            | Test::Between(v, _) => v,
        }
    }

    /// Mirror of `Condition::is_met_by`.
    #[inline]
    fn is_met_by(self, current: Value) -> bool {
        match self {
            Test::Equals(v) => current == v,
            Test::NotEquals(v) => current.same_type(v) && current != v,
            Test::AtLeast(v) => numeric_pair(current, v).is_some_and(|(c, t)| c >= t),
            Test::AtMost(v) => numeric_pair(current, v).is_some_and(|(c, t)| c <= t),
            Test::MoreThan(v) => numeric_pair(current, v).is_some_and(|(c, t)| c > t),
            Test::LessThan(v) => numeric_pair(current, v).is_some_and(|(c, t)| c < t),
            Test::Between(lo, hi) => {
                numeric_pair(current, lo).is_some_and(|(c, l)| c >= l)
                    && numeric_pair(current, hi).is_some_and(|(c, h)| c <= h)
            }
        }
    }

    /// Mirror of `Condition::is_met_by`, `distance` and `direction` in one
    /// dispatch (plan 008, D2): `Met` exactly when `is_met_by` holds,
    /// `Mismatch` exactly when `distance` is `None` (wrong type, or
    /// `Absent`), and otherwise `Unmet` carrying that distance and
    /// direction. `test_mirrors_condition_on_every_variant` checks the
    /// three equivalences.
    #[inline]
    fn evaluate(self, current: Value) -> Evaluation {
        if !current.same_type(self.value()) {
            return Evaluation::Mismatch;
        }
        if self.is_met_by(current) {
            return Evaluation::Met;
        }
        let (distance, direction) = match self {
            Test::Equals(v) => match numeric_pair(current, v) {
                Some((c, t)) if c < t => (c.abs_diff(t), Direction::Increase),
                Some((c, t)) => (c.abs_diff(t), Direction::Decrease),
                None => (1, Direction::Either),
            },
            Test::NotEquals(_) => (1, Direction::Either),
            Test::AtLeast(v) | Test::MoreThan(v) => {
                let Some((c, t)) = numeric_pair(current, v) else {
                    return Evaluation::Mismatch;
                };
                let strict = matches!(self, Test::MoreThan(_));
                (
                    t.abs_diff(c).saturating_add(u64::from(strict)),
                    Direction::Increase,
                )
            }
            Test::AtMost(v) | Test::LessThan(v) => {
                let Some((c, t)) = numeric_pair(current, v) else {
                    return Evaluation::Mismatch;
                };
                let strict = matches!(self, Test::LessThan(_));
                (
                    c.abs_diff(t).saturating_add(u64::from(strict)),
                    Direction::Decrease,
                )
            }
            Test::Between(lo, hi) => {
                let (Some((c, l)), Some((_, h))) =
                    (numeric_pair(current, lo), numeric_pair(current, hi))
                else {
                    return Evaluation::Mismatch;
                };
                if c < l {
                    (l.abs_diff(c), Direction::Increase)
                } else {
                    (c.abs_diff(h), Direction::Decrease)
                }
            }
        };
        Evaluation::Unmet {
            distance,
            direction,
        }
    }
}

/// How a value stands against a [`Test`] (see [`Test::evaluate`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Evaluation {
    /// The condition holds.
    Met,
    /// The condition does not hold; `distance` raw units in `direction`
    /// would satisfy it.
    Unmet { distance: u64, direction: Direction },
    /// The value has the wrong type (or is `Absent`), so no distance exists.
    Mismatch,
}

/// A [`StateOperation`] compiled over [`Value`]s. `apply` mirrors
/// `State::apply` case by case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Effect {
    Set(Value),
    Add(Value),
    Subtract(Value),
}

impl Effect {
    fn compile<'a>(operation: &'a StateOperation, strings: &mut Interner<'a>) -> Effect {
        match operation {
            StateOperation::Set(v) => Effect::Set(Value::compile(v, strings)),
            StateOperation::Add(v) => Effect::Add(Value::compile(v, strings)),
            StateOperation::Subtract(v) => Effect::Subtract(Value::compile(v, strings)),
        }
    }

    /// Mirror of one iteration of `State::apply`: a `Set` replaces the value
    /// whatever its type; a delta applies to a present value of the same
    /// type, or to `Absent` as the typed zero, and otherwise does nothing.
    fn apply(self, current: &mut Value) {
        match self {
            Effect::Set(value) => *current = value,
            Effect::Add(delta) | Effect::Subtract(delta) => {
                let subtract = matches!(self, Effect::Subtract(_));
                let next = match (*current, delta) {
                    (Value::Absent, Value::I64(d)) => Some(Value::I64(shift(0, d, subtract))),
                    (Value::I64(c), Value::I64(d)) => Some(Value::I64(shift(c, d, subtract))),
                    (Value::Absent, Value::F64(d)) => Some(Value::F64(shift(0, d, subtract))),
                    (Value::F64(c), Value::F64(d)) => Some(Value::F64(shift(c, d, subtract))),
                    // Type mismatch or non-numeric delta: no effect.
                    _ => None,
                };
                if let Some(value) = next {
                    *current = value;
                }
            }
        }
    }

    /// Mirror of `StateOperation::signed_delta`.
    fn signed_delta(self) -> Option<i64> {
        match self {
            Effect::Set(_) => None,
            Effect::Add(v) => v.numeric().map(|(_, n)| n),
            Effect::Subtract(v) => v.numeric().map(|(_, n)| n.saturating_neg()),
        }
    }
}

/// Assigns dense indices to strings in first-seen order. Borrows the strings
/// from the problem being compiled and keeps owned copies for debugging.
#[derive(Default)]
struct Interner<'a> {
    index: HashMap<&'a str, usize>,
    items: Vec<String>,
}

impl<'a> Interner<'a> {
    fn intern(&mut self, text: &'a str) -> usize {
        *self.index.entry(text).or_insert_with(|| {
            self.items.push(text.to_string());
            self.items.len() - 1
        })
    }
}

/// `(variable name, item)` pairs of a map in name order, so that compiled
/// problems do not depend on `HashMap` iteration order (plan 007, D6).
fn sorted_by_name<'a, T>(
    pairs: impl Iterator<Item = (&'a String, &'a T)>,
) -> Vec<(&'a String, &'a T)> {
    let mut pairs: Vec<_> = pairs.collect();
    pairs.sort_by_key(|(name, _)| *name);
    pairs
}

/// A live action compiled to slots. `preconditions` and `effects` are in
/// variable-name order.
struct CompiledAction {
    cost: Cost,
    preconditions: Vec<(usize, Test)>,
    effects: Vec<(usize, Effect)>,
}

impl CompiledAction {
    /// Mirror of `Action::can_execute`.
    fn can_execute(&self, state: &[Value]) -> bool {
        is_met(&self.preconditions, state)
    }

    /// Mirror of `State::apply` with this action's effects, in place.
    fn apply(&self, state: &mut [Value]) {
        for &(slot, effect) in &self.effects {
            effect.apply(&mut state[slot]);
        }
    }

    /// The effect on `slot`, if any.
    fn effect_on(&self, slot: usize) -> Option<Effect> {
        self.effects
            .iter()
            .find(|(s, _)| *s == slot)
            .map(|(_, effect)| *effect)
    }
}

/// Mirror of `Conditions::is_met_by` over compiled conditions.
fn is_met(conditions: &[(usize, Test)], state: &[Value]) -> bool {
    conditions
        .iter()
        .all(|&(slot, test)| test.is_met_by(state[slot]))
}

/// The problem handed to `plan`, compiled once per call to slot-indexed form
/// (plan 007, D1). Only variables the goal or a live action mentions get a
/// slot (D3); slots are assigned in first-mention order with every map
/// walked in variable-name order (D6).
struct Compiled {
    /// Slot -> variable name. Not read by the search; the test-only
    /// `encode` uses it to probe the heuristic with arbitrary states.
    #[cfg_attr(not(test), allow(dead_code))]
    names: Vec<String>,
    /// `Value::Str` id -> text. Same use as `names`.
    #[cfg_attr(not(test), allow(dead_code))]
    strings: Vec<String>,
    /// The initial state, one value per slot.
    initial: Vec<Value>,
    goal: Vec<(usize, Test)>,
    /// Aligned with the live actions handed to `new`.
    actions: Vec<CompiledAction>,
}

impl Compiled {
    /// `costs` is aligned with `actions`.
    fn new<'a>(
        initial_state: &'a State,
        goal: &'a Goal,
        actions: &[&'a Action],
        costs: &[Cost],
    ) -> Self {
        debug_assert_eq!(actions.len(), costs.len());
        let mut slots = Interner::default();
        let mut strings = Interner::default();

        let goal = sorted_by_name(goal.conditions.iter())
            .into_iter()
            .map(|(name, condition)| (slots.intern(name), Test::compile(condition, &mut strings)))
            .collect();

        let actions = actions
            .iter()
            .zip(costs)
            .map(|(action, &cost)| CompiledAction {
                cost,
                preconditions: sorted_by_name(action.preconditions.iter())
                    .into_iter()
                    .map(|(name, condition)| {
                        (slots.intern(name), Test::compile(condition, &mut strings))
                    })
                    .collect(),
                effects: sorted_by_name(action.effects.iter())
                    .into_iter()
                    .map(|(name, operation)| {
                        (slots.intern(name), Effect::compile(operation, &mut strings))
                    })
                    .collect(),
            })
            .collect();

        let initial = slots
            .items
            .iter()
            .map(|name| {
                initial_state
                    .get_var(name)
                    .map_or(Value::Absent, |var| Value::compile(var, &mut strings))
            })
            .collect();

        Compiled {
            names: slots.items,
            strings: strings.items,
            initial,
            goal,
            actions,
        }
    }

    /// Encodes `state` over this problem's slots, interning any string the
    /// compilation has not seen. For tests, which probe the heuristic with
    /// states other than the initial one.
    #[cfg(test)]
    fn encode(&mut self, state: &State) -> Vec<Value> {
        self.names
            .iter()
            .map(|name| match state.get_var(name) {
                None => Value::Absent,
                Some(StateVar::Bool(b)) => Value::Bool(*b),
                Some(StateVar::I64(n)) => Value::I64(*n),
                Some(StateVar::F64(n)) => Value::F64(*n),
                Some(StateVar::String(s)) => {
                    let id = match self.strings.iter().position(|known| known == s) {
                        Some(id) => id,
                        None => {
                            self.strings.push(s.clone());
                            self.strings.len() - 1
                        }
                    };
                    Value::Str(id as u32)
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Heuristic
// ---------------------------------------------------------------------------

/// Everything the heuristic needs to know about one fact (a goal condition
/// or a precondition), computed once per `plan()` call from the action set.
struct ConditionInfo {
    slot: usize,
    test: Test,
    /// Cheapest cost per unit of increase among `Add`/`Subtract` effects on
    /// this variable (`None` if no action increases it).
    increase_rate: Option<StepRate>,
    /// Cheapest cost per unit of decrease.
    decrease_rate: Option<StepRate>,
    /// Cheapest action that changes this variable in any way
    /// (`Cost::UNREACHABLE` if none does).
    min_touch_cost: Cost,
    /// Indices of the actions that have any effect on this variable.
    touchers: Vec<usize>,
    /// Some action steps this variable (`increase_rate` or `decrease_rate`
    /// is set), so `lower_bound` depends on the current value.
    has_rate: bool,
    /// The part of `lower_bound` that does not depend on the current value
    /// (plan 008, D1): the cheapest `Set` to a satisfying value, or `Set` to
    /// another value plus steps from there. `None` if no `Set` can help.
    set_bound: Option<Cost>,
}

impl ConditionInfo {
    /// The cheapest rate for moving the variable in `direction`, or `None`
    /// if no action moves it that way. `Satisfied` needs no movement.
    #[inline]
    fn rate_for(&self, direction: Direction) -> Option<StepRate> {
        match direction {
            Direction::Satisfied => Some(StepRate {
                cost: Cost::ZERO,
                per_unit: 1,
            }),
            Direction::Increase => self.increase_rate,
            Direction::Decrease => self.decrease_rate,
            Direction::Either => match (self.increase_rate, self.decrease_rate) {
                (Some(a), Some(b)) => Some(a.cheaper(b)),
                (a, b) => a.or(b),
            },
        }
    }

    /// A lower bound on the cost of the steps that take `from` to a value
    /// satisfying the condition, or `None` if no step action moves it the
    /// right way. Zero when `from` already satisfies it.
    #[inline]
    fn steps_from(&self, from: Value) -> Option<Cost> {
        match self.test.evaluate(from) {
            Evaluation::Met => Some(Cost::ZERO),
            Evaluation::Unmet {
                distance,
                direction,
            } => self.rate_for(direction).map(|rate| rate.price(distance)),
            Evaluation::Mismatch => None,
        }
    }

    /// Computes `set_bound` from the distinct `Set` values and their cheapest
    /// costs, once the rates are known.
    fn set_bound_from(&self, sets: &HashMap<Value, Cost>) -> Option<Cost> {
        sets.iter()
            .filter_map(|(&value, &cost)| {
                self.steps_from(value)
                    .map(|steps| cost.saturating_add(steps))
            })
            .min()
    }

    /// A lower bound on the cost of any action sequence that takes this
    /// variable from `current` to a value satisfying the condition, or `None`
    /// if no such sequence can exist.
    ///
    /// Every sequence of effects on one variable is equivalent to "optionally
    /// one final `Set`, then some `Add`/`Subtract` steps" (any earlier `Set`
    /// is overwritten). So the bound is the cheapest of:
    ///
    /// 1. a `Set` directly to a satisfying value;
    /// 2. a `Set` to a non-satisfying value, plus steps from there;
    /// 3. steps from the current value.
    ///
    /// Steps are priced at `distance * best cost-per-unit`, the continuous
    /// relaxation, which never exceeds the cost of any real mix of step
    /// actions. Preconditions are ignored, which can only make the real cost
    /// higher. The result is also at least `min_touch_cost`, since at least
    /// one action must change this variable.
    ///
    /// Cases 1 and 2 are `set_bound`, precomputed; only case 3 reads
    /// `current`, and only when some action steps this variable at all, so
    /// a boolean or string fact costs one comparison and a load.
    #[inline]
    fn lower_bound(&self, current: Value) -> Option<Cost> {
        let mut best = self.set_bound;
        if self.has_rate {
            // `Add`/`Subtract` treat a missing variable as zero (see
            // `State::apply`), so steps from a missing variable are priced
            // from the typed zero. Non-numeric conditions on a missing
            // variable can only be met by a `Set`, covered by `set_bound`.
            let steps = match current {
                Value::Absent => self
                    .test
                    .value()
                    .numeric_zero()
                    .and_then(|zero| self.steps_from(zero)),
                present => self.steps_from(present),
            };
            if let Some(steps) = steps {
                best = Some(best.map_or(steps, |b| b.min(steps)));
            }
        }
        best.map(|b| b.max(self.min_touch_cost))
    }
}

// ---------------------------------------------------------------------------
// Relaxed reachability
// ---------------------------------------------------------------------------

/// Over-approximation of the values one variable can ever hold, ignoring
/// delete effects and treating numeric movement as unbounded. Anything this
/// says is impossible really is impossible; the converse does not hold.
#[derive(Debug, Default, Clone)]
struct Possible {
    /// Values the variable can be set to (initial value plus `Set` effects of
    /// possibly applicable actions). Empty means the variable is never
    /// present.
    base: HashSet<StateVar>,
    /// Some possibly applicable action adds to this variable.
    can_increase: bool,
    /// Some possibly applicable action subtracts from this variable.
    can_decrease: bool,
}

impl Possible {
    /// True when some possible value satisfies `cond`.
    ///
    /// `Add`/`Subtract` only move a variable that is already present, so the
    /// flags are consulted relative to the base values: from any base value
    /// the variable may move without limit in each enabled direction.
    fn may_satisfy(&self, cond: &Condition) -> bool {
        self.base.iter().any(|value| {
            if cond.is_met_by(value) {
                return true;
            }
            // Flags only mean something for numeric variables. Type
            // validation already guarantees `cond` and `value` agree.
            if !matches!(value, StateVar::I64(_) | StateVar::F64(_)) {
                return false;
            }
            match cond.direction(value) {
                Direction::Satisfied => true,
                Direction::Increase => self.can_increase,
                Direction::Decrease => self.can_decrease,
                Direction::Either => self.can_increase || self.can_decrease,
            }
        })
    }
}

/// Result of the relaxed reachability analysis over an action set.
struct Reachability {
    vars: HashMap<String, Possible>,
    /// Indices (into the caller's slice) of the actions that can possibly
    /// ever fire, in their original order.
    applicable: Vec<usize>,
}

impl Reachability {
    /// Iterates to a fixpoint: an action is possibly applicable when every
    /// precondition intersects the possible values of its variable; its
    /// effects then widen those values. Possible sets only grow, so this
    /// terminates in at most `actions.len() + 1` rounds.
    fn analyse(initial: &State, actions: &[Action]) -> Self {
        let mut vars: HashMap<String, Possible> = HashMap::new();
        for (key, value) in initial.iter() {
            vars.entry(key.clone())
                .or_default()
                .base
                .insert(value.clone());
        }

        let mut is_applicable = vec![false; actions.len()];
        let mut changed = true;
        while changed {
            changed = false;
            for (idx, action) in actions.iter().enumerate() {
                if is_applicable[idx] {
                    continue;
                }
                let ok = action
                    .preconditions
                    .iter()
                    .all(|(key, cond)| vars.get(key).is_some_and(|p| p.may_satisfy(cond)));
                if !ok {
                    continue;
                }
                is_applicable[idx] = true;
                changed = true;
                for (key, op) in &action.effects {
                    let entry = vars.entry(key.clone()).or_default();
                    match op {
                        StateOperation::Set(value) => {
                            entry.base.insert(value.clone());
                        }
                        StateOperation::Add(delta) | StateOperation::Subtract(delta) => {
                            // A missing variable counts as zero when added to
                            // (see `State::apply`). Nothing ever removes a
                            // variable, so it can be missing exactly when the
                            // initial state lacks it; zero then joins its
                            // possible values no matter what else may `Set` it.
                            if !initial.contains(key)
                                && let Some(zero) = delta.numeric_zero()
                            {
                                entry.base.insert(zero);
                            }
                            match op.signed_delta() {
                                Some(d) if d > 0 => entry.can_increase = true,
                                Some(d) if d < 0 => entry.can_decrease = true,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        let applicable = (0..actions.len()).filter(|&i| is_applicable[i]).collect();
        Reachability { vars, applicable }
    }

    /// False only when some goal condition can never be satisfied by any
    /// reachable value of its variable.
    fn goal_may_be_reachable(&self, goal: &Goal) -> bool {
        goal.conditions
            .iter()
            .all(|(key, cond)| self.vars.get(key).is_some_and(|p| p.may_satisfy(cond)))
    }
}

/// Which bound a goal condition contributes to the greedy sum, and what set
/// of actions that bound charges (see `GoalAnalysis::estimate`).
#[derive(Clone, Copy, Debug)]
enum Candidate {
    /// `ConditionInfo::lower_bound` of goal fact `i`; charges the actions
    /// that touch its variable.
    Effects(usize),
    /// The precondition-aware `h` of goal fact `i`; charges the closure of
    /// its touchers under "touchers of my preconditions' variables".
    Chain(usize),
}

impl Candidate {
    /// Row of this candidate in `GoalAnalysis::compatible`.
    fn row(self) -> usize {
        match self {
            Candidate::Effects(i) => 2 * i,
            Candidate::Chain(i) => 2 * i + 1,
        }
    }
}

/// A fixed-size set of small integers (facts, or candidate rows) stored as
/// machine words (plan 008, D4). Sized once per `plan()` call; no operation
/// allocates.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    const WORD_BITS: usize = u64::BITS as usize;

    /// An empty set that can hold `0..len`.
    fn new(len: usize) -> Self {
        BitSet {
            words: vec![0; len.div_ceil(Self::WORD_BITS)],
        }
    }

    fn insert(&mut self, index: usize) {
        self.words[index / Self::WORD_BITS] |= 1 << (index % Self::WORD_BITS);
    }

    fn remove(&mut self, index: usize) {
        self.words[index / Self::WORD_BITS] &= !(1 << (index % Self::WORD_BITS));
    }

    fn contains(&self, index: usize) -> bool {
        self.words[index / Self::WORD_BITS] & (1 << (index % Self::WORD_BITS)) != 0
    }

    fn clear(&mut self) {
        self.words.fill(0);
    }

    /// Makes `self` equal to `other`, which must have the same capacity.
    fn copy_from(&mut self, other: &BitSet) {
        self.words.copy_from_slice(&other.words);
    }

    /// Every member of `self` is a member of `other` (same capacity).
    fn is_subset_of(&self, other: &BitSet) -> bool {
        self.words
            .iter()
            .zip(&other.words)
            .all(|(mine, theirs)| mine & !theirs == 0)
    }

    /// The members of `self`, ascending.
    fn iter(&self) -> Members<'_> {
        Members {
            words: &self.words,
            mask: None,
            next_word: 0,
            bits: 0,
            base: 0,
        }
    }

    /// The members of `self` that are not members of `other`, ascending.
    fn iter_without<'a>(&'a self, other: &'a BitSet) -> Members<'a> {
        Members {
            words: &self.words,
            mask: Some(&other.words),
            next_word: 0,
            bits: 0,
            base: 0,
        }
    }
}

/// Iterator over the members of a [`BitSet`], optionally minus a mask.
/// Hand-written so that the hot loops of `estimate` compile to a word load,
/// `trailing_zeros` and a clear-lowest-bit per member.
struct Members<'a> {
    words: &'a [u64],
    mask: Option<&'a [u64]>,
    /// Index of the next word to load.
    next_word: usize,
    /// Remaining bits of the current word.
    bits: u64,
    /// Index of bit 0 of the current word.
    base: usize,
}

impl Iterator for Members<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        while self.bits == 0 {
            let word = *self.words.get(self.next_word)?;
            self.bits = match self.mask {
                Some(mask) => word & !mask[self.next_word],
                None => word,
            };
            self.base = self.next_word * BitSet::WORD_BITS;
            self.next_word += 1;
        }
        let bit = self.bits.trailing_zeros() as usize;
        self.bits &= self.bits - 1;
        Some(self.base + bit)
    }
}

/// Working memory for `GoalAnalysis::estimate`, kept between calls so that a
/// call allocates nothing.
struct Scratch {
    /// The facts the current state satisfies.
    satisfied: BitSet,
    /// The facts whose `h` is final: `satisfied` plus every fact popped so
    /// far.
    done: BitSet,
    /// Facts with a finite `h` that have not been popped yet (the Dijkstra
    /// frontier).
    pending: BitSet,
    /// Per unmet fact: `lower_bound` from the current state
    /// (`Cost::UNREACHABLE` = none). Unspecified for satisfied facts.
    lower_bound: Vec<Cost>,
    /// Per fact: the precondition-aware bound (`Cost::UNREACHABLE` = ∞).
    h: Vec<Cost>,
    /// `(bound, candidate)` for the unmet goal facts.
    candidates: Vec<(Cost, Candidate)>,
    /// Candidate rows chosen by the mixed greedy and by the effects-only
    /// greedy (see `estimate`).
    chosen_mixed: BitSet,
    chosen_effects: BitSet,
}

/// Precomputed data for the heuristic, built once per `plan()` call from
/// the goal and the live action set.
///
/// A *fact* is a `(variable, condition)` pair: every goal condition and
/// every precondition of every live action, deduplicated. Goal facts come
/// first in `facts`.
struct GoalAnalysis {
    facts: Vec<ConditionInfo>,
    /// Number of goal facts (`facts[..goal_count]`).
    goal_count: usize,
    /// Per live action: its cost.
    costs: Vec<Cost>,
    /// Per live action: the facts it requires.
    precondition_facts: Vec<BitSet>,
    /// Per live action: the facts whose variable it changes.
    touched_facts: Vec<BitSet>,
    /// Per fact: the actions that require it.
    required_by: Vec<Vec<usize>>,
    /// `(fact, slot)` for every fact, grouped by the kind of test so that
    /// the dispatch in `Test::is_met_by` follows a predictable pattern.
    classify_order: Vec<(usize, usize)>,
    /// `compatible[c]` holds `d` when the charged action sets of candidates
    /// `c` and `d` are disjoint, so their bounds may be summed. Candidate
    /// `Effects(i)` is row `2 * i`, `Chain(i)` is row `2 * i + 1` (see
    /// `Candidate::row`).
    compatible: Vec<BitSet>,
    scratch: Scratch,
}

impl GoalAnalysis {
    fn new(compiled: &Compiled) -> Self {
        let actions = &compiled.actions;

        // Collect the facts: goal conditions first, then preconditions.
        let mut facts: Vec<ConditionInfo> = Vec::new();
        let mut fact_index: HashMap<(usize, Test), usize> = HashMap::new();
        let mut intern = |slot: usize, test: Test, facts: &mut Vec<ConditionInfo>| {
            *fact_index.entry((slot, test)).or_insert_with(|| {
                facts.push(ConditionInfo {
                    slot,
                    test,
                    increase_rate: None,
                    decrease_rate: None,
                    min_touch_cost: Cost::UNREACHABLE,
                    touchers: Vec::new(),
                    has_rate: false,
                    set_bound: None,
                });
                facts.len() - 1
            })
        };
        for &(slot, test) in &compiled.goal {
            intern(slot, test, &mut facts);
        }
        let goal_count = facts.len();
        let preconditions: Vec<Vec<usize>> = actions
            .iter()
            .map(|action| {
                action
                    .preconditions
                    .iter()
                    .map(|&(slot, test)| intern(slot, test, &mut facts))
                    .collect()
            })
            .collect();
        drop(fact_index);

        let mut classify_order: Vec<(usize, usize)> = facts
            .iter()
            .enumerate()
            .map(|(f, info)| (f, info.slot))
            .collect();
        classify_order.sort_by_key(|&(f, slot)| (facts[f].test.kind(), slot, f));

        // Effect summary per fact.
        for info in &mut facts {
            let mut set_costs: HashMap<Value, Cost> = HashMap::new();
            for (idx, action) in actions.iter().enumerate() {
                let Some(effect) = action.effect_on(info.slot) else {
                    continue;
                };
                let cost = action.cost;
                info.touchers.push(idx);
                info.min_touch_cost = info.min_touch_cost.min(cost);
                match effect {
                    Effect::Set(value) => {
                        set_costs
                            .entry(value)
                            .and_modify(|c| *c = (*c).min(cost))
                            .or_insert(cost);
                    }
                    Effect::Add(_) | Effect::Subtract(_) => {
                        let Some(signed) = effect.signed_delta() else {
                            continue;
                        };
                        if signed == 0 {
                            continue;
                        }
                        let rate = StepRate {
                            cost,
                            per_unit: signed.unsigned_abs(),
                        };
                        let slot = if signed > 0 {
                            &mut info.increase_rate
                        } else {
                            &mut info.decrease_rate
                        };
                        *slot = Some(slot.map_or(rate, |r| r.cheaper(rate)));
                    }
                }
            }
            // Both rates are final here, so the `Set`-based part of the
            // bound can be priced once (D1).
            info.has_rate = info.increase_rate.is_some() || info.decrease_rate.is_some();
            info.set_bound = info.set_bound_from(&set_costs);
        }

        let mut touched_facts: Vec<BitSet> = vec![BitSet::new(facts.len()); actions.len()];
        for (f, info) in facts.iter().enumerate() {
            for &a in &info.touchers {
                touched_facts[a].insert(f);
            }
        }
        let mut precondition_facts: Vec<BitSet> = vec![BitSet::new(facts.len()); actions.len()];
        let mut required_by: Vec<Vec<usize>> = vec![Vec::new(); facts.len()];
        for (a, pre) in preconditions.iter().enumerate() {
            for &f in pre {
                precondition_facts[a].insert(f);
                required_by[f].push(a);
            }
        }

        // Charged action sets, as bitmaps over the live actions: row `2 * i`
        // for `Effects(i)`, row `2 * i + 1` for `Chain(i)`.
        let mut charged: Vec<Vec<bool>> = Vec::with_capacity(2 * goal_count);
        for i in 0..goal_count {
            let mut effects = vec![false; actions.len()];
            for &a in &facts[i].touchers {
                effects[a] = true;
            }
            // Closure: touchers of the fact's variable, then touchers of
            // every variable in their preconditions, transitively.
            let mut chain = effects.clone();
            let mut seen_fact = vec![false; facts.len()];
            let mut worklist = vec![i];
            seen_fact[i] = true;
            while let Some(f) = worklist.pop() {
                for &a in &facts[f].touchers {
                    chain[a] = true;
                    for &p in &preconditions[a] {
                        if !seen_fact[p] {
                            seen_fact[p] = true;
                            worklist.push(p);
                        }
                    }
                }
            }
            charged.push(effects);
            charged.push(chain);
        }
        let n = charged.len();
        let mut compatible = vec![BitSet::new(n); n];
        for c in 0..n {
            for d in 0..n {
                if c != d && charged[c].iter().zip(&charged[d]).all(|(x, y)| !(*x && *y)) {
                    compatible[c].insert(d);
                }
            }
        }

        let scratch = Scratch {
            satisfied: BitSet::new(facts.len()),
            done: BitSet::new(facts.len()),
            pending: BitSet::new(facts.len()),
            lower_bound: vec![Cost::UNREACHABLE; facts.len()],
            h: vec![Cost::UNREACHABLE; facts.len()],
            candidates: Vec::with_capacity(n),
            chosen_mixed: BitSet::new(n),
            chosen_effects: BitSet::new(n),
        };

        GoalAnalysis {
            facts,
            goal_count,
            costs: actions.iter().map(|action| action.cost).collect(),
            precondition_facts,
            touched_facts,
            required_by,
            classify_order,
            compatible,
            scratch,
        }
    }

    /// Admissible estimate of the remaining cost from `state` to the goal.
    ///
    /// Returns `None` when some goal condition can never be satisfied from
    /// `state` with the available actions. Returns `Some(Cost::ZERO)`
    /// exactly when the goal is already satisfied.
    ///
    /// Two bounds are computed per unmet goal condition:
    ///
    /// 1. the effect bound (`ConditionInfo::lower_bound`), which charges the
    ///    actions touching the condition's variable;
    /// 2. the chain bound `h`, an `h_max` over facts: `h(f)` is the minimum
    ///    over touchers `a` of `H(pre(a)) + max(cost(a), lower_bound(f))`,
    ///    where `H` is the maximum `h` of `a`'s preconditions. The first
    ///    action to touch `f`'s variable has its preconditions satisfied by
    ///    actions that do not touch it, so the two parts never charge the
    ///    same action occurrence. `h` charges the transitive closure of
    ///    touchers.
    ///
    /// Bounds whose charged sets are pairwise disjoint are summed, greedily
    /// largest first; the result is the larger of the greedy over both kinds
    /// and the greedy over effect bounds alone, so it never falls below
    /// either the old estimate or the largest single `h`.
    fn estimate(&mut self, state: &[Value]) -> Option<Cost> {
        let scratch = &mut self.scratch;
        let facts = &self.facts;

        // Classify. Satisfied facts are final at zero; unmet ones get their
        // effect bound and start at infinity.
        scratch.satisfied.clear();
        let mut unmet_goal_facts = 0usize;
        for &(f, slot) in &self.classify_order {
            let current = state[slot];
            let info = &facts[f];
            if info.test.is_met_by(current) {
                scratch.satisfied.insert(f);
                scratch.h[f] = Cost::ZERO;
            } else {
                scratch.lower_bound[f] = info.lower_bound(current).unwrap_or(Cost::UNREACHABLE);
                scratch.h[f] = Cost::UNREACHABLE;
                if f < self.goal_count {
                    unmet_goal_facts += 1;
                }
            }
        }
        if unmet_goal_facts == 0 {
            return Some(Cost::ZERO);
        }

        // Dijkstra over facts. An action becomes applicable when all its
        // precondition facts are final; `H` is then the value of the last
        // one, which is the maximum because pops are monotone. Relaxing
        // visits only the touched facts that are not final yet; an
        // unreachable `lower_bound` saturates the candidate to infinity, so
        // it never beats the current `h`.
        scratch.done.copy_from(&scratch.satisfied);
        scratch.pending.clear();
        let relax = |scratch: &mut Scratch, a: usize, enabling: Cost| {
            for g in self.touched_facts[a].iter_without(&scratch.done) {
                let candidate = enabling.saturating_add(self.costs[a].max(scratch.lower_bound[g]));
                if candidate < scratch.h[g] {
                    scratch.h[g] = candidate;
                    scratch.pending.insert(g);
                }
            }
        };
        for (a, required) in self.precondition_facts.iter().enumerate() {
            if required.is_subset_of(&scratch.satisfied) {
                relax(scratch, a, Cost::ZERO);
            }
        }
        let mut goal_facts_left = unmet_goal_facts;
        loop {
            // Pop: the pending fact with the smallest `h`, lowest index on a
            // tie. The frontier is a few facts, so a scan beats a heap.
            let mut popped: Option<(Cost, usize)> = None;
            for f in scratch.pending.iter() {
                let value = scratch.h[f];
                if popped.is_none_or(|(best, _)| value < best) {
                    popped = Some((value, f));
                }
            }
            let Some((value, f)) = popped else {
                break;
            };
            scratch.pending.remove(f);
            scratch.done.insert(f);
            if f < self.goal_count {
                goal_facts_left -= 1;
                if goal_facts_left == 0 {
                    break;
                }
            }
            for &a in &self.required_by[f] {
                // `f` was the last unmet precondition of `a` exactly when
                // all of them are final now; every earlier pop of one of
                // them failed this test, so `a` relaxes once.
                if self.precondition_facts[a].is_subset_of(&scratch.done) {
                    relax(scratch, a, value);
                }
            }
        }

        // Candidates for the unmet goal facts.
        scratch.candidates.clear();
        for i in 0..self.goal_count {
            if scratch.satisfied.contains(i) {
                continue;
            }
            let h = scratch.h[i];
            if h == Cost::UNREACHABLE {
                return None;
            }
            let lb = scratch.lower_bound[i];
            if lb == Cost::UNREACHABLE {
                return None;
            }
            if unmet_goal_facts == 1 {
                // One unmet goal fact: `h >= lb` always (`h` adds an
                // enabling cost to `max(cost, lb)`) and the two candidates
                // share the fact's touchers, so both greedies below would
                // pick one candidate each and the larger is `h`.
                return Some(h);
            }
            scratch.candidates.push((lb, Candidate::Effects(i)));
            scratch.candidates.push((h, Candidate::Chain(i)));
        }
        scratch
            .candidates
            .sort_by_key(|&(bound, _)| std::cmp::Reverse(bound));

        // Greedy over all candidates and, in the same sweep, over effect
        // bounds alone. The two chosen sets are independent, so this is the
        // same as two passes; the sort is stable, so ties keep insertion
        // order (`Effects(i)` before `Chain(i)`, lower `i` first).
        scratch.chosen_mixed.clear();
        scratch.chosen_effects.clear();
        let mut mixed = Cost::ZERO;
        let mut effects = Cost::ZERO;
        for &(bound, candidate) in &scratch.candidates {
            let c = candidate.row();
            let compatible = &self.compatible[c];
            if scratch.chosen_mixed.is_subset_of(compatible) {
                scratch.chosen_mixed.insert(c);
                mixed = mixed.saturating_add(bound);
            }
            if matches!(candidate, Candidate::Effects(_))
                && scratch.chosen_effects.is_subset_of(compatible)
            {
                scratch.chosen_effects.insert(c);
                effects = effects.saturating_add(bound);
            }
        }
        Some(mixed.max(effects))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conditions::{IntoCondition, at_least, at_most};

    /// The heuristic over a compiled problem, probed with `State`s.
    struct Probe {
        compiled: Compiled,
        analysis: GoalAnalysis,
    }

    impl Probe {
        fn estimate(&mut self, state: &State) -> Option<Cost> {
            let values = self.compiled.encode(state);
            self.analysis.estimate(&values)
        }
    }

    /// Builds the heuristic over `actions` the way `plan` does.
    fn analysis_for(goal: &Goal, actions: &[Action]) -> Probe {
        let refs: Vec<&Action> = actions.iter().collect();
        let costs = validate_costs(actions).expect("test costs are valid");
        let compiled = Compiled::new(&State::new(), goal, &refs, &costs);
        let analysis = GoalAnalysis::new(&compiled);
        Probe { compiled, analysis }
    }

    /// `Cost` from a float, for readable expectations.
    fn cost(value: f64) -> Cost {
        Cost::try_from_f64(value).expect("test cost is valid")
    }

    #[test]
    fn cost_conversion_rounds_and_rejects_unrepresentable_values() {
        assert_eq!(Cost::try_from_f64(0.1), Some(Cost(100)));
        assert_eq!(Cost::try_from_f64(1.23456), Some(Cost(1235)));
        assert_eq!(Cost::try_from_f64(0.0), Some(Cost::ZERO));
        let max = Cost::try_from_f64(StateVar::MAX_F64).expect("MAX_F64 is representable");
        assert!(max > Cost(9_223_372_036_854_000_000) && max.0 <= i64::MAX as u64);
        assert_eq!(Cost::try_from_f64(StateVar::MAX_F64 * 1.001), None);
        assert_eq!(Cost::try_from_f64(-0.0004), Some(Cost::ZERO)); // rounds to zero
        assert_eq!(Cost::try_from_f64(-0.001), None);
        assert_eq!(Cost::try_from_f64(f64::NAN), None);
        assert_eq!(Cost::try_from_f64(f64::INFINITY), None);
        assert_eq!(Cost::try_from_f64(1e16), None);
        assert_eq!(Cost(1000).to_f64(), 1.0);
        assert_eq!(Cost::UNREACHABLE.saturating_add(Cost(1)), Cost::UNREACHABLE);
    }

    #[test]
    fn step_rate_compares_exact_fractions_and_rounds_prices_down() {
        // 1 per 3 units vs 2 per 7 units: 1/3 > 2/7, so the second is cheaper.
        let a = StepRate {
            cost: Cost(1000),
            per_unit: 3,
        };
        let b = StepRate {
            cost: Cost(2000),
            per_unit: 7,
        };
        assert!(b.is_cheaper_than(&a));
        assert!(!a.is_cheaper_than(&b));
        assert!(!a.is_cheaper_than(&a));
        assert_eq!(a.cheaper(b).per_unit, 7);
        assert_eq!(b.cheaper(a).per_unit, 7);

        // 10 units at 1000 per 3 units = 3333.33 -> floor 3333.
        assert_eq!(a.price(10), Cost(3333));
        assert_eq!(a.price(0), Cost::ZERO);
        // Saturates instead of overflowing.
        let huge = StepRate {
            cost: Cost(u64::MAX),
            per_unit: 1,
        };
        assert_eq!(huge.price(u64::MAX), Cost::UNREACHABLE);

        // The `u64` fast path and the `u128` fallback agree (plan 008, D3):
        // products on both sides of 2^64, exact and inexact divisions.
        let reference = |rate: &StepRate, distance: u64| {
            let exact = (distance as u128) * (rate.cost.0 as u128) / (rate.per_unit as u128);
            Cost(exact.min(u64::MAX as u128) as u64)
        };
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for _ in 0..10_000 {
            let magnitude = next() % 64;
            let rate = StepRate {
                cost: Cost(next() >> (next() % 64)),
                per_unit: (next() >> (next() % 64)).max(1),
            };
            let distance = next() >> magnitude;
            assert_eq!(
                rate.price(distance),
                reference(&rate, distance),
                "{rate:?} x {distance}"
            );
        }
        let wide = StepRate {
            cost: Cost(u64::MAX / 2),
            per_unit: 4,
        };
        assert_eq!(wide.price(4), Cost(u64::MAX / 2)); // product overflows u64
        assert_eq!(wide.price(3), Cost(((u64::MAX / 2) as u128 * 3 / 4) as u64));
    }

    /// `BitSet` across a word boundary: membership, subset, iteration with
    /// and without a mask (plan 008, D4).
    #[test]
    fn bitset_tracks_bits_across_words() {
        let mut set = BitSet::new(130);
        assert_eq!(set.words.len(), 3);
        for index in [0, 5, 63, 64, 100, 127, 128, 129] {
            set.insert(index);
        }
        assert!(set.contains(63) && set.contains(64) && set.contains(129));
        assert!(!set.contains(1) && !set.contains(65) && !set.contains(126));
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![0, 5, 63, 64, 100, 127, 128, 129]
        );

        set.remove(64);
        set.remove(129);
        assert!(!set.contains(64) && !set.contains(129));
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![0, 5, 63, 100, 127, 128]
        );

        let mut mask = BitSet::new(130);
        mask.insert(5);
        mask.insert(127);
        mask.insert(1); // not in `set`; irrelevant to the difference
        assert_eq!(
            set.iter_without(&mask).collect::<Vec<_>>(),
            vec![0, 63, 100, 128]
        );
        assert!(!mask.is_subset_of(&set));
        mask.remove(1);
        assert!(mask.is_subset_of(&set));
        assert!(!set.is_subset_of(&mask));
        assert!(BitSet::new(130).is_subset_of(&mask));

        let mut copy = BitSet::new(130);
        copy.copy_from(&set);
        assert_eq!(copy, set);
        copy.clear();
        assert_eq!(copy.iter().count(), 0);
        assert!(copy.is_subset_of(&set));
    }

    #[test]
    fn open_entry_orders_by_lowest_f_then_highest_g() {
        let a = OpenEntry {
            f: Cost(10),
            g: Cost(1),
            id: 0,
        };
        let b = OpenEntry {
            f: Cost(5),
            g: Cost(1),
            id: 1,
        };
        let c = OpenEntry {
            f: Cost(5),
            g: Cost(4),
            id: 2,
        };
        assert!(b > a);
        assert!(c > b);

        let mut heap = BinaryHeap::new();
        heap.push(a);
        heap.push(b);
        heap.push(c);
        assert_eq!(heap.pop().unwrap().id, 2);
        assert_eq!(heap.pop().unwrap().id, 1);
        assert_eq!(heap.pop().unwrap().id, 0);
    }

    #[test]
    fn estimate_is_zero_when_satisfied_and_none_when_unreachable() {
        let goal = Goal::builder("g")
            .requires("gold", at_least(100))
            .requires("has_key", true)
            .build();
        let actions = [
            Action::builder("work").cost(2.0).adds("gold", 10).build(),
            Action::builder("key")
                .cost(5.0)
                .sets("has_key", true)
                .build(),
        ];
        let mut analysis = analysis_for(&goal, &actions);

        let done = State::builder()
            .set("gold", 150)
            .set("has_key", true)
            .build();
        assert_eq!(analysis.estimate(&done), Some(Cost::ZERO));

        // 100 gold at 0.2 per unit = 20, key = 5; disjoint, so summed.
        let start = State::builder()
            .set("gold", 0)
            .set("has_key", false)
            .build();
        assert_eq!(analysis.estimate(&start), Some(cost(25.0)));

        // Nothing sets a missing "has_key" if the key action is removed.
        let mut analysis = analysis_for(&goal, &actions[..1]);
        assert_eq!(analysis.estimate(&start), None);
    }

    #[test]
    fn lower_bound_prices_steps_from_zero_for_a_missing_variable() {
        // `Add` on a missing variable starts from zero, so at_least(10) with a
        // cost-1 `adds(2)` needs 5 steps.
        let goal = Goal::builder("g").requires("count", at_least(10)).build();
        let actions = [Action::builder("step").cost(1.0).adds("count", 2).build()];
        let mut analysis = analysis_for(&goal, &actions);
        assert_eq!(analysis.estimate(&State::new()), Some(cost(5.0)));

        // Same for floats, in fixed-point units: 2.5 at 0.5 per cost 1 = 5.
        let goal = Goal::builder("g").requires("xp", at_least(2.5)).build();
        let actions = [Action::builder("step").cost(1.0).adds("xp", 0.5).build()];
        let mut analysis = analysis_for(&goal, &actions);
        assert_eq!(analysis.estimate(&State::new()), Some(cost(5.0)));

        // A missing bool can only be Set; with no Set action it is unreachable.
        let goal = Goal::builder("g").requires("flag", true).build();
        let actions = [Action::builder("step").cost(1.0).adds("count", 2).build()];
        let mut analysis = analysis_for(&goal, &actions);
        assert_eq!(analysis.estimate(&State::new()), None);
    }

    #[test]
    fn lower_bound_is_exact_for_fractional_rates() {
        // 7 units at cost 1 per 3 units: the continuous bound is 2.333..,
        // stored as 2333 thousandths. Three real steps cost 3.0, so the
        // bound is admissible and tighter than any float rounding would be.
        let goal = Goal::builder("g").requires("x", at_least(7)).build();
        let actions = [Action::builder("step").cost(1.0).adds("x", 3).build()];
        let mut analysis = analysis_for(&goal, &actions);
        let start = State::builder().set("x", 0).build();
        assert_eq!(analysis.estimate(&start), Some(Cost(2333)));
    }

    #[test]
    fn estimate_accounts_for_cheap_set_before_steps() {
        // Set to 90 (cost 1) then +1 per cost 1 is 11, far below 100 * 1.
        let goal = Goal::builder("g").requires("gold", at_least(100)).build();
        let actions = [
            Action::builder("jump").cost(1.0).sets("gold", 90).build(),
            Action::builder("step").cost(1.0).adds("gold", 1).build(),
        ];
        let mut analysis = analysis_for(&goal, &actions);
        let start = State::builder().set("gold", 0).build();
        assert_eq!(analysis.estimate(&start), Some(cost(11.0)));
    }

    #[test]
    fn estimate_uses_max_for_overlapping_conditions() {
        // One action satisfies both conditions, so their bounds must not be
        // summed.
        let goal = Goal::builder("g")
            .requires("a", true)
            .requires("b", true)
            .build();
        let actions = [Action::builder("both")
            .cost(3.0)
            .sets("a", true)
            .sets("b", true)
            .build()];
        let mut analysis = analysis_for(&goal, &actions);
        let start = State::builder().set("a", false).set("b", false).build();
        assert_eq!(analysis.estimate(&start), Some(cost(3.0)));
    }

    fn possible(values: &[StateVar], inc: bool, dec: bool) -> Possible {
        Possible {
            base: values.iter().cloned().collect(),
            can_increase: inc,
            can_decrease: dec,
        }
    }

    #[test]
    fn possible_may_satisfy_covers_every_condition_kind() {
        use crate::conditions::{at_most, between, less_than, more_than, not};
        let ten = StateVar::I64(10);

        // Fixed at 10, no movement.
        let fixed = possible(std::slice::from_ref(&ten), false, false);
        assert!(fixed.may_satisfy(&10.into_condition()));
        assert!(!fixed.may_satisfy(&11.into_condition()));
        assert!(fixed.may_satisfy(&at_least(10)));
        assert!(!fixed.may_satisfy(&at_least(11)));
        assert!(!fixed.may_satisfy(&more_than(10)));
        assert!(fixed.may_satisfy(&at_most(10)));
        assert!(!fixed.may_satisfy(&less_than(10)));
        assert!(fixed.may_satisfy(&between(5, 15)));
        assert!(!fixed.may_satisfy(&between(11, 15)));
        assert!(!fixed.may_satisfy(&not(10)));

        // Can only increase from 10.
        let up = possible(std::slice::from_ref(&ten), true, false);
        assert!(up.may_satisfy(&11.into_condition()));
        assert!(!up.may_satisfy(&9.into_condition()));
        assert!(up.may_satisfy(&more_than(1000)));
        assert!(!up.may_satisfy(&less_than(10)));
        assert!(up.may_satisfy(&between(20, 30)));
        assert!(!up.may_satisfy(&between(0, 5)));
        assert!(up.may_satisfy(&not(10)));

        // Can only decrease from 10.
        let down = possible(std::slice::from_ref(&ten), false, true);
        assert!(down.may_satisfy(&9.into_condition()));
        assert!(!down.may_satisfy(&11.into_condition()));
        assert!(down.may_satisfy(&less_than(-5)));
        assert!(!down.may_satisfy(&at_least(11)));
        assert!(down.may_satisfy(&between(0, 5)));
        assert!(!down.may_satisfy(&between(20, 30)));

        // Two base values, no movement.
        let two = possible(&[StateVar::I64(1), StateVar::I64(7)], false, false);
        assert!(two.may_satisfy(&between(5, 9)));
        assert!(two.may_satisfy(&not(1)));
        assert!(!two.may_satisfy(&at_least(8)));

        // Never present.
        assert!(!possible(&[], true, true).may_satisfy(&at_least(0)));

        // Bool: flags are meaningless and must not be consulted.
        let flag = possible(&[StateVar::Bool(false)], true, true);
        assert!(!flag.may_satisfy(&true.into_condition()));
        assert!(flag.may_satisfy(&false.into_condition()));
    }

    #[test]
    fn reachability_fixpoint_chains_through_preconditions() {
        let initial = State::builder()
            .set("a", false)
            .set("b", false)
            .set("gold", 0)
            .build();
        let actions = vec![
            // dead: requires c, which nothing sets
            Action::builder("dead")
                .requires("c", true)
                .sets("b", true)
                .build(),
            // live only after `first`
            Action::builder("second")
                .requires("a", true)
                .sets("b", true)
                .build(),
            Action::builder("first").sets("a", true).build(),
            Action::builder("earn")
                .requires("b", true)
                .adds("gold", 5)
                .build(),
        ];
        let reach = Reachability::analyse(&initial, &actions);
        assert_eq!(reach.applicable, vec![1, 2, 3]);
        assert!(reach.vars["gold"].can_increase);
        assert!(!reach.vars["gold"].can_decrease);
        assert!(!reach.vars.contains_key("c"));

        let ok = Goal::builder("g").requires("gold", at_least(1_000)).build();
        assert!(reach.goal_may_be_reachable(&ok));
        let bad = Goal::builder("g").requires("gold", at_most(-1)).build();
        assert!(!reach.goal_may_be_reachable(&bad));
        let missing = Goal::builder("g").requires("c", true).build();
        assert!(!reach.goal_may_be_reachable(&missing));
    }

    #[test]
    fn type_mismatch_is_reported_before_search() {
        let planner = Planner::new();
        let initial = State::builder().set("value", 0).build();
        let goal = Goal::builder("g").requires("value", "string").build();
        let err = planner.plan(&initial, &goal, &[]).unwrap_err();
        assert_eq!(
            err,
            PlannerError::IncompatibleStateTypes {
                variable: "value".to_string(),
                expected: StateType::I64,
                expected_origin: VariableOrigin::InitialState,
                found: StateType::String,
                found_origin: VariableOrigin::Goal("g".to_string()),
            }
        );
    }

    #[test]
    fn negative_cost_is_rejected() {
        let planner = Planner::new();
        let initial = State::builder().set("x", 0).build();
        let goal = Goal::builder("g").requires("x", 1).build();
        let actions = vec![Action::builder("free").cost(-1.0).build()];
        assert_eq!(
            planner.plan(&initial, &goal, &actions),
            Err(PlannerError::InvalidActionCost {
                action: "free".to_string(),
                cost: -1.0,
            })
        );
    }

    // Plan 006 (BUG-022)

    fn step(state: &State, action: &Action) -> State {
        assert!(action.can_execute(state));
        action.apply_effect(state)
    }

    /// A `Set` chain is priced at its length, and shrinks as it is walked.
    #[test]
    fn estimate_prices_precondition_chains() {
        let goal = Goal::builder("g").requires("s3", true).build();
        let actions = [
            Action::builder("step1")
                .requires("s0", true)
                .sets("s1", true)
                .build(),
            Action::builder("step2")
                .requires("s1", true)
                .sets("s2", true)
                .build(),
            Action::builder("step3")
                .requires("s2", true)
                .sets("s3", true)
                .build(),
        ];
        let mut analysis = analysis_for(&goal, &actions);
        let start = State::builder()
            .set("s0", true)
            .set("s1", false)
            .set("s2", false)
            .set("s3", false)
            .build();
        assert_eq!(analysis.estimate(&start), Some(cost(3.0)));
        let after_one = step(&start, &actions[0]);
        assert_eq!(analysis.estimate(&after_one), Some(cost(2.0)));
        let after_two = step(&after_one, &actions[1]);
        assert_eq!(analysis.estimate(&after_two), Some(cost(1.0)));
        assert_eq!(
            analysis.estimate(&step(&after_two, &actions[2])),
            Some(Cost::ZERO)
        );
    }

    /// Two goal conditions enabled by the same action: the chain bounds
    /// overlap on `get_tool`, so they are not summed with each other. The
    /// chain bound of one plus the effect bound of the other charges disjoint
    /// sets, and that sum is the true cost, 12.
    #[test]
    fn estimate_does_not_double_count_shared_enablers() {
        let goal = Goal::builder("g")
            .requires("a", true)
            .requires("b", true)
            .build();
        let actions = [
            Action::builder("get_tool")
                .cost(10.0)
                .sets("tool", true)
                .build(),
            Action::builder("set_a")
                .requires("tool", true)
                .sets("a", true)
                .build(),
            Action::builder("set_b")
                .requires("tool", true)
                .sets("b", true)
                .build(),
        ];
        let mut analysis = analysis_for(&goal, &actions);
        let start = State::builder()
            .set("tool", false)
            .set("a", false)
            .set("b", false)
            .build();
        // Chain(a) = 10 + 1 charges {get_tool, set_a}; Chain(b) overlaps on
        // get_tool and is skipped; Effects(b) = 1 charges {set_b} and is
        // added: 12, never 22.
        assert_eq!(analysis.estimate(&start), Some(cost(12.0)));
        // With the tool in hand the chains collapse to the effect bounds,
        // which are disjoint and sum to 2.
        let with_tool = step(&start, &actions[0]);
        assert_eq!(analysis.estimate(&with_tool), Some(cost(2.0)));
    }

    /// An action that consumes what it requires (`craft` needs metal and
    /// spends it) is a cycle in the fact graph; Dijkstra finalises each
    /// fact once and the bound is exact on this two-step problem.
    #[test]
    fn estimate_handles_self_enabling_cycles() {
        let goal = Goal::builder("g").requires("parts", at_least(1)).build();
        let actions = [
            Action::builder("mine").cost(3.0).adds("metal", 5).build(),
            Action::builder("craft")
                .cost(2.0)
                .requires("metal", at_least(5))
                .subtracts("metal", 5)
                .adds("parts", 1)
                .build(),
        ];
        let mut analysis = analysis_for(&goal, &actions);
        let start = State::builder().set("metal", 0).set("parts", 0).build();
        assert_eq!(analysis.estimate(&start), Some(cost(5.0)));
        assert_eq!(
            analysis.estimate(&step(&start, &actions[0])),
            Some(cost(2.0))
        );
    }

    /// Dead ends the one-shot reachability pass cannot see: once `gold`
    /// passes the exact value `unlock` needs and nothing lowers it, the goal
    /// is unreachable from that state and the estimate says so.
    #[test]
    fn estimate_prunes_dead_ends_the_reachability_pass_cannot_see() {
        let goal = Goal::builder("g").requires("has_key", true).build();
        let actions = [
            Action::builder("work").cost(1.0).adds("gold", 2).build(),
            Action::builder("unlock")
                .requires("gold", 5)
                .sets("has_key", true)
                .build(),
        ];
        let mut analysis = analysis_for(&goal, &actions);
        let start = State::builder()
            .set("gold", 0)
            .set("has_key", false)
            .build();
        // 5 units at 0.5 per unit = 2.5, plus the unlock.
        assert_eq!(analysis.estimate(&start), Some(cost(3.5)));
        let past = State::builder()
            .set("gold", 6)
            .set("has_key", false)
            .build();
        assert_eq!(analysis.estimate(&past), None);
    }

    // Plan 007 (compiled problem)

    /// `Test` agrees with `Condition` on every variant, for every kind of
    /// current value including a type mismatch, and fails on `Absent`.
    #[test]
    fn test_mirrors_condition_on_every_variant() {
        use crate::conditions::{between, less_than, more_than, not};
        let conditions = [
            at_least(10),
            at_most(10),
            more_than(10),
            less_than(10),
            between(5, 10),
            10.into_condition(),
            not(10),
            at_least(1.5),
            between(0.5, 1.5),
            1.5.into_condition(),
            not(1.5),
            true.into_condition(),
            not(true),
            "a".into_condition(),
            not("a"),
        ];
        let mut currents: Vec<StateVar> = (-3..15).map(StateVar::I64).collect();
        currents.extend((-3..15).map(|n| StateVar::F64(n * 250)));
        currents.extend([
            StateVar::Bool(true),
            StateVar::Bool(false),
            StateVar::String("a".into()),
            StateVar::String("b".into()),
        ]);

        let mut strings = Interner::default();
        let mut unmet_seen = 0;
        for condition in &conditions {
            let test = Test::compile(condition, &mut strings);
            assert!(!test.is_met_by(Value::Absent), "{condition}");
            assert_eq!(
                test.evaluate(Value::Absent),
                Evaluation::Mismatch,
                "{condition}"
            );
            for current in &currents {
                let value = Value::compile(current, &mut strings);
                assert_eq!(
                    test.is_met_by(value),
                    condition.is_met_by(current),
                    "{condition} vs {current}"
                );
                // `evaluate` folds `is_met_by`, `distance` and `direction`
                // into one answer (plan 008, D2).
                let expected = if condition.is_met_by(current) {
                    Evaluation::Met
                } else if let Some(distance) = condition.distance(current) {
                    unmet_seen += 1;
                    Evaluation::Unmet {
                        distance,
                        direction: condition.direction(current),
                    }
                } else {
                    Evaluation::Mismatch
                };
                assert_eq!(test.evaluate(value), expected, "{condition} vs {current}");
            }
        }
        assert!(unmet_seen > 100, "the matrix exercises the Unmet arm");
        // Both "a" and "b" were interned exactly once.
        assert_eq!(strings.items, vec!["a".to_string(), "b".to_string()]);
    }

    /// `Effect::apply` agrees with `State::apply` for every combination of
    /// operation and current value, including the mismatch and saturation
    /// cases.
    #[test]
    fn effect_mirrors_state_apply() {
        let operations = [
            StateOperation::set(7),
            StateOperation::set(2.5),
            StateOperation::set(true),
            StateOperation::set("town"),
            StateOperation::add(3),
            StateOperation::subtract(3),
            StateOperation::add(0.25),
            StateOperation::subtract(0.25),
            StateOperation::add(i64::MAX),
            StateOperation::subtract(i64::MAX),
            StateOperation::Add(StateVar::Bool(true)),
        ];
        let currents: [Option<StateVar>; 7] = [
            None,
            Some(StateVar::I64(10)),
            Some(StateVar::I64(i64::MAX - 1)),
            Some(StateVar::F64(-1_000)),
            Some(StateVar::F64(i64::MIN + 1)),
            Some(StateVar::Bool(false)),
            Some(StateVar::String("home".into())),
        ];
        let mut strings = Interner::default();
        let effects: Vec<Effect> = operations
            .iter()
            .map(|operation| Effect::compile(operation, &mut strings))
            .collect();
        // Encodes with the same string ids as the compiled effects.
        let mut table = strings.items.clone();
        let mut encode = |var: Option<&StateVar>| match var {
            None => Value::Absent,
            Some(StateVar::Bool(b)) => Value::Bool(*b),
            Some(StateVar::I64(n)) => Value::I64(*n),
            Some(StateVar::F64(n)) => Value::F64(*n),
            Some(StateVar::String(text)) => {
                let id = table
                    .iter()
                    .position(|known| known == text)
                    .unwrap_or_else(|| {
                        table.push(text.clone());
                        table.len() - 1
                    });
                Value::Str(id as u32)
            }
        };
        for (operation, effect) in operations.iter().zip(&effects) {
            for current in &currents {
                let mut state = State::new();
                if let Some(var) = current {
                    state.set("x", var.clone());
                }
                let changes = HashMap::from([("x".to_string(), operation.clone())]);
                state.apply(&changes);
                let expected = encode(state.get_var("x"));

                let mut value = encode(current.as_ref());
                effect.apply(&mut value);
                assert_eq!(value, expected, "{operation} on {current:?}");
            }
        }
    }

    /// Variables nothing reads or writes get no slot; variables a condition
    /// reads but nothing provides are `Absent` (D3).
    #[test]
    fn compiled_drops_inert_variables_and_keeps_absent_ones() {
        let initial = State::builder()
            .set("inert", 42)
            .set("gold", 5)
            .set("location", "home")
            .build();
        let goal = Goal::builder("g").requires("has_key", true).build();
        let actions = [Action::builder("unlock")
            .requires("gold", at_least(5))
            .requires("missing", true)
            .sets("has_key", true)
            .sets("location", "vault")
            .build()];
        let refs: Vec<&Action> = actions.iter().collect();
        let costs = validate_costs(&actions).unwrap();
        let compiled = Compiled::new(&initial, &goal, &refs, &costs);

        assert_eq!(
            compiled.names,
            vec!["has_key", "gold", "missing", "location"]
        );
        assert_eq!(compiled.strings, vec!["vault", "home"]);
        assert_eq!(
            compiled.initial,
            vec![Value::Absent, Value::I64(5), Value::Absent, Value::Str(1),]
        );
        assert_eq!(compiled.goal, vec![(0, Test::Equals(Value::Bool(true)))]);
        let unlock = &compiled.actions[0];
        assert_eq!(unlock.cost, Cost(1000));
        assert_eq!(
            unlock.preconditions,
            vec![
                (1, Test::AtLeast(Value::I64(5))),
                (2, Test::Equals(Value::Bool(true))),
            ]
        );
        assert_eq!(
            unlock.effects,
            vec![
                (0, Effect::Set(Value::Bool(true))),
                (3, Effect::Set(Value::Str(0))),
            ]
        );
        assert!(!unlock.can_execute(&compiled.initial));
    }

    /// Goal conditions, preconditions and effects are compiled in variable
    /// name order whatever order they were added in (D6).
    #[test]
    fn compiled_orders_by_variable_name() {
        fn slots<T>(pairs: &[(usize, T)]) -> Vec<usize> {
            pairs.iter().map(|(slot, _)| *slot).collect()
        }
        let goal = Goal::builder("g")
            .requires("zeta", true)
            .requires("alpha", true)
            .requires("mid", true)
            .build();
        let actions = [Action::builder("a")
            .requires("zeta", false)
            .requires("beta", false)
            .sets("zeta", true)
            .sets("alpha", true)
            .sets("beta", true)
            .build()];
        let refs: Vec<&Action> = actions.iter().collect();
        let costs = validate_costs(&actions).unwrap();
        for _ in 0..3 {
            let compiled = Compiled::new(&State::new(), &goal, &refs, &costs);
            assert_eq!(compiled.names, vec!["alpha", "mid", "zeta", "beta"]);
            assert_eq!(slots(&compiled.goal), vec![0, 1, 2]);
            assert_eq!(slots(&compiled.actions[0].preconditions), vec![3, 2]);
            assert_eq!(slots(&compiled.actions[0].effects), vec![0, 3, 2]);
        }
    }

    /// The arena and its table: every committed state is found again, a
    /// state that differs in one slot is not, discarded children leave no
    /// trace, and the table grows through several resizes.
    #[test]
    fn search_graph_finds_duplicates_and_survives_resize() {
        let width = 3;
        let initial = [Value::I64(0), Value::Bool(false), Value::Str(0)];
        let mut graph = SearchGraph::new(&initial, Cost::ZERO);
        assert_eq!(graph.find_child_of(&initial), Some(0));

        let count = 5_000;
        for i in 1..=count {
            graph.begin_child(0)[0] = Value::I64(i);
            let (hash, existing) = graph.find_child();
            assert_eq!(existing, None, "state {i} is new");
            let id = graph.commit_child(hash, Cost(i as u64), Cost::ZERO, Some((0, 0)));
            assert_eq!(id, i as usize);
        }
        assert_eq!(graph.nodes.len(), count as usize + 1);
        assert_eq!(graph.arena.len(), graph.nodes.len() * width);
        assert!(graph.table.len() >= 2 * graph.nodes.len());
        assert!(graph.table.len().is_power_of_two());

        for i in 0..=count {
            let state = [Value::I64(i), Value::Bool(false), Value::Str(0)];
            assert_eq!(graph.find_child_of(&state), Some(i as usize));
            assert_eq!(graph.state(i as usize), &state);
            assert_eq!(graph.nodes[i as usize].g, Cost(i as u64));
        }
        // One slot different, or a different type in the same slot.
        assert_eq!(
            graph.find_child_of(&[Value::I64(7), Value::Bool(true), Value::Str(0)]),
            None
        );
        assert_eq!(
            graph.find_child_of(&[Value::F64(7), Value::Bool(false), Value::Str(0)]),
            None
        );
        assert_eq!(
            graph.find_child_of(&[Value::I64(7), Value::Bool(false), Value::Str(1)]),
            None
        );
        assert_eq!(graph.arena.len(), graph.nodes.len() * width);
    }

    impl SearchGraph {
        /// Looks `state` up as if it were a freshly generated child.
        fn find_child_of(&mut self, state: &[Value]) -> Option<usize> {
            self.begin_child(0).copy_from_slice(state);
            let (_, existing) = self.find_child();
            self.discard_child();
            existing
        }
    }

    /// Planning the same problem repeatedly gives the same plan: nothing in
    /// the search depends on `HashMap` iteration order any more.
    #[test]
    fn plan_is_deterministic_across_calls() {
        let initial = State::builder()
            .set("a", false)
            .set("b", false)
            .set("c", false)
            .set("gold", 0)
            .build();
        // Three equally priced ways to earn gold, two of which also flip a
        // flag the goal wants: plenty of ties for the heap and the greedy.
        let goal = Goal::builder("g")
            .requires("a", true)
            .requires("b", true)
            .requires("c", true)
            .requires("gold", at_least(6))
            .build();
        let actions = [
            Action::builder("earn_a")
                .adds("gold", 2)
                .sets("a", true)
                .build(),
            Action::builder("earn_b")
                .adds("gold", 2)
                .sets("b", true)
                .build(),
            Action::builder("earn").adds("gold", 2).build(),
            Action::builder("set_c").sets("c", true).build(),
        ];
        let planner = Planner::new();
        let first = planner.plan(&initial, &goal, &actions).unwrap();
        for _ in 0..20 {
            let again = planner.plan(&initial, &goal, &actions).unwrap();
            assert_eq!(again, first);
        }
        assert_eq!(first.cost, 4.0);
    }

    /// The new estimate is never below the effect-only greedy sum.
    #[test]
    fn estimate_is_at_least_the_effect_only_bound() {
        // Two disjoint goal conditions with no preconditions anywhere: the
        // chain bound equals the effect bound and the sum is kept.
        let goal = Goal::builder("g")
            .requires("gold", at_least(100))
            .requires("has_key", true)
            .build();
        let actions = [
            Action::builder("work").cost(2.0).adds("gold", 10).build(),
            Action::builder("key")
                .cost(5.0)
                .sets("has_key", true)
                .build(),
        ];
        let mut analysis = analysis_for(&goal, &actions);
        let start = State::builder()
            .set("gold", 0)
            .set("has_key", false)
            .build();
        assert_eq!(analysis.estimate(&start), Some(cost(25.0)));
    }
}
