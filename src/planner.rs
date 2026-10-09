use crate::actions::Action;
use crate::conditions::{Condition, Direction};
use crate::goals::Goal;
use crate::state::{State, StateOperation, StateType, StateVar, fixed_point_from_f64};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::rc::Rc;

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
    fn price(&self, distance: u64) -> Cost {
        let exact = (distance as u128) * (self.cost.0 as u128) / (self.per_unit as u128);
        Cost(exact.min(u64::MAX as u128) as u64)
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
        let mut analysis = GoalAnalysis::new(goal, &live, &live_costs);

        let Some(initial_h) = analysis.estimate(initial_state) else {
            return Err(PlannerError::NoPlanFound);
        };

        let mut nodes: Vec<Node> = Vec::new();
        let mut index: HashMap<Rc<State>, usize> = HashMap::new();
        let mut open = BinaryHeap::new();

        let initial = Rc::new(initial_state.clone());
        index.insert(Rc::clone(&initial), 0);
        nodes.push(Node {
            state: initial,
            g: Cost::ZERO,
            parent: None,
        });
        open.push(OpenEntry {
            f: initial_h,
            g: Cost::ZERO,
            id: 0,
        });

        let mut expanded = 0usize;
        while let Some(OpenEntry { g, id, .. }) = open.pop() {
            // A cheaper path to this node was found after this entry was
            // pushed; the newer entry will handle it.
            if g > nodes[id].g {
                continue;
            }

            if goal.is_satisfied(&nodes[id].state) {
                return Ok(reconstruct_path(&nodes, &live, &reach.applicable, id));
            }

            if let Some(limit) = self.max_expansions
                && expanded >= limit
            {
                return Err(PlannerError::SearchLimitExceeded { expanded });
            }
            expanded += 1;

            for (action_idx, action) in live.iter().enumerate() {
                if !action.can_execute(&nodes[id].state) {
                    continue;
                }
                let next_state = action.apply_effect(&nodes[id].state);
                let tentative_g = g.saturating_add(live_costs[action_idx]);

                let existing = index.get(&next_state).copied();
                if let Some(next_id) = existing
                    && tentative_g >= nodes[next_id].g
                {
                    continue;
                }

                let Some(h) = analysis.estimate(&next_state) else {
                    continue;
                };

                let next_id = match existing {
                    Some(next_id) => {
                        nodes[next_id].g = tentative_g;
                        nodes[next_id].parent = Some((id, action_idx));
                        next_id
                    }
                    None => {
                        let next_id = nodes.len();
                        let state = Rc::new(next_state);
                        index.insert(Rc::clone(&state), next_id);
                        nodes.push(Node {
                            state,
                            g: tentative_g,
                            parent: Some((id, action_idx)),
                        });
                        next_id
                    }
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

/// A discovered state in the search graph.
struct Node {
    state: Rc<State>,
    /// Cheapest known cost from the initial state to this node.
    g: Cost,
    /// `(parent node, index of the action that led here)`.
    parent: Option<(usize, usize)>,
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
// Heuristic
// ---------------------------------------------------------------------------

/// Everything the heuristic needs to know about one goal condition, computed
/// once per `plan()` call from the action set.
struct ConditionInfo {
    key: String,
    condition: Condition,
    /// Distinct values that some action can `Set` this variable to, with the
    /// cheapest cost that does so.
    sets: Vec<(StateVar, Cost)>,
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
    /// `lower_bound` when it does not depend on the current value: no action
    /// steps this variable, so only `Set`s matter. `None` while the rates are
    /// still being collected, or when a rate exists.
    fixed_bound: Option<Option<Cost>>,
}

impl ConditionInfo {
    /// The cheapest rate for moving the variable in `direction`, or `None`
    /// if no action moves it that way. `Satisfied` needs no movement.
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
    fn lower_bound(&self, current: Option<&StateVar>) -> Option<Cost> {
        if let Some(fixed) = self.fixed_bound {
            return fixed;
        }
        let mut best: Option<Cost> = None;
        let mut consider = |candidate: Cost| {
            best = Some(best.map_or(candidate, |b| b.min(candidate)));
        };

        for (value, cost) in &self.sets {
            if self.condition.is_met_by(value) {
                consider(*cost);
            } else if let Some(d) = self.condition.distance(value)
                && let Some(rate) = self.rate_for(self.condition.direction(value))
            {
                consider(cost.saturating_add(rate.price(d)));
            }
        }

        // `Add`/`Subtract` treat a missing variable as zero (see
        // `State::apply`), so steps from a missing variable are priced from
        // the typed zero. Non-numeric conditions on a missing variable can
        // only be met by a `Set`, handled above.
        let zero;
        let start = match current {
            Some(cur) => Some(cur),
            None => {
                zero = self.condition.value().numeric_zero();
                zero.as_ref()
            }
        };
        if let Some(cur) = start
            && let Some(d) = self.condition.distance(cur)
            && let Some(rate) = self.rate_for(self.condition.direction(cur))
        {
            consider(rate.price(d));
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

/// Working memory for `GoalAnalysis::estimate`, kept between calls so that a
/// call allocates nothing.
#[derive(Default)]
struct Scratch {
    /// Per fact: does the current state satisfy it?
    satisfied: Vec<bool>,
    /// Per unmet fact: `lower_bound` from the current state (`None` =
    /// unreachable). Unspecified for satisfied facts.
    lower_bound: Vec<Option<Cost>>,
    /// Per fact: the precondition-aware bound (`Cost::UNREACHABLE` = ∞).
    h: Vec<Cost>,
    /// Per fact: has it been popped from the heap (finalised)?
    done: Vec<bool>,
    /// Per action: precondition facts not yet finalised.
    remaining: Vec<usize>,
    heap: BinaryHeap<std::cmp::Reverse<(Cost, usize)>>,
    /// `(bound, candidate)` for the unmet goal facts.
    candidates: Vec<(Cost, Candidate)>,
    chosen: Vec<usize>,
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
    preconditions: Vec<Vec<usize>>,
    /// Per live action: the facts whose variable it changes.
    touched: Vec<Vec<usize>>,
    /// Per fact: the actions that require it.
    required_by: Vec<Vec<usize>>,
    /// Distinct variable names, and per variable the facts on it, so that a
    /// state is probed once per variable rather than once per fact.
    variables: Vec<String>,
    variable_facts: Vec<Vec<usize>>,
    /// `compatible[c][d]`: the charged action sets of candidates `c` and `d`
    /// are disjoint, so their bounds may be summed. Candidate `Effects(i)` is
    /// row `2 * i`, `Chain(i)` is row `2 * i + 1` (see `Candidate::row`).
    compatible: Vec<Vec<bool>>,
    scratch: Scratch,
}

impl GoalAnalysis {
    /// `costs` is aligned with `actions`.
    fn new<'a>(goal: &'a Goal, actions: &[&'a Action], costs: &[Cost]) -> Self {
        debug_assert_eq!(actions.len(), costs.len());

        // Collect the facts: goal conditions first, then preconditions. The
        // index borrows keys and conditions from `goal` and `actions`.
        let mut facts: Vec<ConditionInfo> = Vec::new();
        let mut fact_index: HashMap<(&'a str, &'a Condition), usize> = HashMap::new();
        let mut intern =
            |key: &'a str, condition: &'a Condition, facts: &mut Vec<ConditionInfo>| {
                *fact_index.entry((key, condition)).or_insert_with(|| {
                    facts.push(ConditionInfo {
                        key: key.to_string(),
                        condition: condition.clone(),
                        sets: Vec::new(),
                        increase_rate: None,
                        decrease_rate: None,
                        min_touch_cost: Cost::UNREACHABLE,
                        touchers: Vec::new(),
                        fixed_bound: None,
                    });
                    facts.len() - 1
                })
            };
        for (key, condition) in goal.conditions.iter() {
            intern(key, condition, &mut facts);
        }
        let goal_count = facts.len();
        let preconditions: Vec<Vec<usize>> = actions
            .iter()
            .map(|action| {
                action
                    .preconditions
                    .iter()
                    .map(|(key, condition)| intern(key, condition, &mut facts))
                    .collect()
            })
            .collect();
        drop(fact_index);

        let mut variables: Vec<String> = Vec::new();
        let mut variable_facts: Vec<Vec<usize>> = Vec::new();
        for (f, info) in facts.iter().enumerate() {
            match variables.iter().position(|v| *v == info.key) {
                Some(v) => variable_facts[v].push(f),
                None => {
                    variables.push(info.key.clone());
                    variable_facts.push(vec![f]);
                }
            }
        }

        // Effect summary per fact, as before, now for every fact.
        for info in &mut facts {
            let mut set_costs: HashMap<StateVar, Cost> = HashMap::new();
            for (idx, (action, &cost)) in actions.iter().zip(costs).enumerate() {
                let Some(op) = action.effects.get(&info.key) else {
                    continue;
                };
                info.touchers.push(idx);
                info.min_touch_cost = info.min_touch_cost.min(cost);
                match op {
                    StateOperation::Set(value) => {
                        set_costs
                            .entry(value.clone())
                            .and_modify(|c| *c = (*c).min(cost))
                            .or_insert(cost);
                    }
                    StateOperation::Add(_) | StateOperation::Subtract(_) => {
                        let Some(signed) = op.signed_delta() else {
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
            info.sets = set_costs.into_iter().collect();
            if info.increase_rate.is_none() && info.decrease_rate.is_none() {
                // Without step rates the bound only looks at `sets`, which
                // does not depend on the current value (`None` is as good a
                // stand-in as any; see the body of `lower_bound`).
                info.fixed_bound = Some(info.lower_bound(None));
            }
        }

        let mut touched: Vec<Vec<usize>> = vec![Vec::new(); actions.len()];
        for (f, info) in facts.iter().enumerate() {
            for &a in &info.touchers {
                touched[a].push(f);
            }
        }
        let mut required_by: Vec<Vec<usize>> = vec![Vec::new(); facts.len()];
        for (a, pre) in preconditions.iter().enumerate() {
            for &f in pre {
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
        let mut compatible = vec![vec![false; n]; n];
        for c in 0..n {
            for d in 0..n {
                compatible[c][d] =
                    c != d && charged[c].iter().zip(&charged[d]).all(|(x, y)| !(*x && *y));
            }
        }

        let scratch = Scratch {
            satisfied: vec![false; facts.len()],
            lower_bound: vec![None; facts.len()],
            h: vec![Cost::UNREACHABLE; facts.len()],
            done: vec![false; facts.len()],
            remaining: vec![0; actions.len()],
            heap: BinaryHeap::new(),
            candidates: Vec::with_capacity(n),
            chosen: Vec::with_capacity(n),
        };

        GoalAnalysis {
            facts,
            goal_count,
            costs: costs.to_vec(),
            preconditions,
            touched,
            required_by,
            variables,
            variable_facts,
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
    fn estimate(&mut self, state: &State) -> Option<Cost> {
        let scratch = &mut self.scratch;
        let facts = &self.facts;

        // Reset and classify, one state lookup per variable.
        scratch.heap.clear();
        let mut unmet_goal_facts = 0usize;
        for (key, on_variable) in self.variables.iter().zip(&self.variable_facts) {
            let current = state.get_var(key);
            for &f in on_variable {
                let info = &facts[f];
                let satisfied = current.is_some_and(|value| info.condition.is_met_by(value));
                scratch.satisfied[f] = satisfied;
                // Satisfied facts are finalised at zero without visiting the
                // heap; they only count towards enabling actions below.
                scratch.done[f] = satisfied;
                scratch.h[f] = if satisfied {
                    Cost::ZERO
                } else {
                    scratch.lower_bound[f] = info.lower_bound(current);
                    if f < self.goal_count {
                        unmet_goal_facts += 1;
                    }
                    Cost::UNREACHABLE
                };
            }
        }
        if unmet_goal_facts == 0 {
            return Some(Cost::ZERO);
        }

        // Dijkstra over facts. An action becomes applicable when all its
        // precondition facts are finalised; `H` is then the value of the
        // last one, which is the maximum because pops are monotone.
        let relax = |scratch: &mut Scratch, a: usize, enabling: Cost| {
            for &g in &self.touched[a] {
                if scratch.satisfied[g] || scratch.done[g] {
                    continue;
                }
                let Some(lb) = scratch.lower_bound[g] else {
                    continue;
                };
                let candidate = enabling.saturating_add(self.costs[a].max(lb));
                if candidate < scratch.h[g] {
                    scratch.h[g] = candidate;
                    scratch.heap.push(std::cmp::Reverse((candidate, g)));
                }
            }
        };
        for (a, pre) in self.preconditions.iter().enumerate() {
            let unmet = pre.iter().filter(|&&f| !scratch.satisfied[f]).count();
            scratch.remaining[a] = unmet;
            if unmet == 0 {
                relax(scratch, a, Cost::ZERO);
            }
        }
        let mut goal_facts_left = unmet_goal_facts;
        while let Some(std::cmp::Reverse((value, f))) = scratch.heap.pop() {
            if scratch.done[f] || value > scratch.h[f] {
                continue;
            }
            scratch.done[f] = true;
            if f < self.goal_count {
                goal_facts_left -= 1;
                if goal_facts_left == 0 {
                    break;
                }
            }
            for &a in &self.required_by[f] {
                scratch.remaining[a] -= 1;
                if scratch.remaining[a] == 0 {
                    relax(scratch, a, value);
                }
            }
        }

        // Candidates for the unmet goal facts.
        scratch.candidates.clear();
        for i in 0..self.goal_count {
            if scratch.satisfied[i] {
                continue;
            }
            let h = scratch.h[i];
            if h == Cost::UNREACHABLE {
                return None;
            }
            let lb = scratch.lower_bound[i]?;
            scratch.candidates.push((lb, Candidate::Effects(i)));
            scratch.candidates.push((h, Candidate::Chain(i)));
        }
        scratch
            .candidates
            .sort_by_key(|&(bound, _)| std::cmp::Reverse(bound));

        // Greedy over all candidates, then over effect bounds alone.
        let compatible = &self.compatible;
        let greedy = |scratch: &mut Scratch, effects_only: bool| -> Cost {
            scratch.chosen.clear();
            let mut total = Cost::ZERO;
            for &(bound, candidate) in &scratch.candidates {
                if effects_only && matches!(candidate, Candidate::Chain(_)) {
                    continue;
                }
                let c = candidate.row();
                if scratch.chosen.iter().all(|&d| compatible[c][d]) {
                    scratch.chosen.push(c);
                    total = total.saturating_add(bound);
                }
            }
            total
        };
        let mixed = greedy(scratch, false);
        let effects = greedy(scratch, true);
        Some(mixed.max(effects))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conditions::{IntoCondition, at_least, at_most};

    /// Builds the heuristic over `actions` the way `plan` does.
    fn analysis_for(goal: &Goal, actions: &[Action]) -> GoalAnalysis {
        let refs: Vec<&Action> = actions.iter().collect();
        let costs = validate_costs(actions).expect("test costs are valid");
        GoalAnalysis::new(goal, &refs, &costs)
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
