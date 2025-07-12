use crate::actions::Action;
use crate::goals::Goal;
use crate::state::State;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::fmt;

/// Errors that can occur during planning.
#[derive(Debug, PartialEq, Eq)]
pub enum PlannerError {
    /// No valid sequence of actions could be found to achieve the goal
    NoPlanFound,
    /// State variables have incompatible types for comparison
    IncompatibleStateTypes(String),
}

impl fmt::Display for PlannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlannerError::NoPlanFound => write!(f, "No plan found"),
            PlannerError::IncompatibleStateTypes(msg) => {
                write!(f, "Incompatible state types: {msg}")
            }
        }
    }
}

impl Error for PlannerError {}

/// A plan represents a sequence of actions that will achieve a goal.
/// It includes the actions to perform and the total cost of execution.
#[derive(Debug)]
pub struct Plan {
    /// The sequence of actions to perform in order
    pub actions: Vec<Action>,
    /// The total cost of executing all actions in the plan
    pub cost: f64,
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

/// A planner that uses A* search to find optimal sequences of actions.
/// The planner is stateless and can be reused for multiple planning requests.
pub struct Planner {}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner {
    /// Creates a new planner instance.
    pub fn new() -> Self {
        Planner {}
    }

    /// Finds a plan to achieve the given goal starting from the initial state.
    ///
    /// Uses A* search algorithm to find the optimal sequence of actions.
    /// Returns a `Plan` containing the actions to perform and their total cost,
    /// or `PlannerError::NoPlanFound` if no valid plan exists.
    ///
    /// # Arguments
    ///
    /// * `initial_state` - The starting state of the world
    /// * `goal` - The goal to achieve
    /// * `actions` - The available actions that can be performed
    pub fn plan(
        &self,
        initial_state: State,
        goal: &Goal,
        actions: &[Action],
    ) -> Result<Plan, PlannerError> {
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut action_taken = HashMap::new();

        g_score.insert(initial_state.clone(), 0.0);
        let initial_h = self.heuristic(&initial_state, &goal.desired_state)?;

        open_set.push(NodeWrapper {
            node: initial_state.clone(),
            f_score: initial_h,
        });

        while let Some(NodeWrapper {
            node: current,
            f_score: _,
        }) = open_set.pop()
        {
            if goal.is_satisfied(&current) {
                let plan = self.reconstruct_path(&came_from, &action_taken, &current);
                return Ok(plan);
            }

            let current_g = *g_score.get(&current).unwrap_or(&f64::INFINITY);
            let transitions = self.get_valid_transitions(&current, actions);

            for (next_state, cost, action) in transitions {
                let tentative_g = current_g + cost;
                let next_h = self.heuristic(&next_state, &goal.desired_state)?;
                let next_f = tentative_g + next_h;

                if tentative_g < *g_score.get(&next_state).unwrap_or(&f64::INFINITY) {
                    came_from.insert(next_state.clone(), current.clone());
                    action_taken.insert(next_state.clone(), action);
                    g_score.insert(next_state.clone(), tentative_g);

                    open_set.push(NodeWrapper {
                        node: next_state,
                        f_score: next_f,
                    });
                }
            }
        }

        Err(PlannerError::NoPlanFound)
    }

    /// Gets all valid transitions from the current state.
    /// Returns a vector of (next_state, cost, action) tuples for actions that can be executed.
    fn get_valid_transitions(
        &self,
        state: &State,
        actions: &[Action],
    ) -> Vec<(State, f64, Action)> {
        let mut transitions = Vec::new();
        for action in actions {
            if action.can_execute(state) {
                let new_state = action.apply_effect(state);
                transitions.push((new_state, action.cost, action.clone()));
            }
        }
        transitions
    }

    /// Calculates the heuristic distance from the current state to the goal state.
    /// This is used by A* to guide the search towards the goal.
    /// Returns the estimated cost to reach the goal from the current state.
    /// Returns an error if state variables have incompatible types.
    fn heuristic(&self, current: &State, goal: &State) -> Result<f64, PlannerError> {
        let mut total_distance = 0;

        // Calculate distance for each goal requirement
        for (key, goal_val) in &goal.vars {
            match current.vars.get(key) {
                Some(current_val) => {
                    let distance = current_val.distance(goal_val).map_err(|_| {
                        PlannerError::IncompatibleStateTypes(format!(
                            "Cannot calculate distance for variable '{key}' due to type mismatch"
                        ))
                    })?;
                    total_distance += distance;
                }
                None => {
                    total_distance += 1; // Penalty for missing keys
                }
            }
        }

        Ok(total_distance as f64)
    }

    /// Reconstructs the final plan from the search data structures.
    /// Traces back through the came_from map to build the sequence of actions.
    fn reconstruct_path(
        &self,
        came_from: &HashMap<State, State>,
        action_taken: &HashMap<State, Action>,
        current: &State,
    ) -> Plan {
        let mut total_cost = 0.0;
        let mut actions = Vec::new();
        let mut current_state = current;

        while let Some(prev_state) = came_from.get(current_state) {
            if let Some(action) = action_taken.get(current_state) {
                actions.push(action.clone());
                total_cost += action.cost;
            }
            current_state = prev_state;
        }

        actions.reverse();
        Plan {
            actions,
            cost: total_cost,
        }
    }
}

/// Wrapper for nodes in the A* search priority queue.
/// Allows states to be ordered by their f-score for efficient retrieval.
#[derive(Clone)]
struct NodeWrapper<N> {
    /// The state being wrapped
    node: N,
    /// The f-score (g + h) used for A* search ordering
    f_score: f64,
}

impl<N: PartialEq> PartialEq for NodeWrapper<N> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<N: Eq> Eq for NodeWrapper<N> {}

impl<N: Eq> Ord for NodeWrapper<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Use total ordering: NaN values are treated as greater than any finite value
        // This means NaN f-scores will have the lowest priority in our min-heap
        other.f_score.total_cmp(&self.f_score)
    }
}

impl<N: Eq> PartialOrd for NodeWrapper<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_wrapper_ordering() {
        let state1 = State::empty();
        let state2 = State::empty();

        let node1 = NodeWrapper {
            node: state1,
            f_score: 10.0,
        };
        let node2 = NodeWrapper {
            node: state2,
            f_score: 5.0,
        };

        // Test ordering - lower f_score should be higher priority
        assert!(node2 > node1);
    }

    #[test]
    fn test_heuristic() {
        let planner = Planner::new();

        let current = State::new().set("value", 0).set("flag", false).build();

        let goal = State::new().set("value", 10).set("flag", true).build();

        let h = planner.heuristic(&current, &goal).unwrap();
        assert!(h > 0.0); // Should have some distance to goal
    }

    #[test]
    fn test_node_wrapper_nan_handling() {
        let state1 = State::empty();
        let state2 = State::empty();
        let state3 = State::empty();

        let normal_node = NodeWrapper {
            node: state1,
            f_score: 10.0,
        };
        let nan_node = NodeWrapper {
            node: state2,
            f_score: f64::NAN,
        };
        let another_nan_node = NodeWrapper {
            node: state3,
            f_score: f64::NAN,
        };

        // Test that NaN nodes are ordered consistently
        // NaN should be treated as the worst score (lowest priority)
        assert!(normal_node > nan_node); // Normal score should beat NaN
        assert_eq!(nan_node.cmp(&another_nan_node), std::cmp::Ordering::Equal); // Two NaN should be equal

        // Test that we can create a BinaryHeap with NaN values without panicking
        let mut heap = std::collections::BinaryHeap::new();
        heap.push(normal_node);
        heap.push(nan_node);
        heap.push(another_nan_node);

        // Should be able to pop without panicking
        let first = heap.pop().unwrap();
        assert_eq!(first.f_score, 10.0); // Normal score should come first
    }

    #[test]
    fn test_heuristic_with_type_mismatch() {
        let planner = Planner::new();

        let current = State::new().set("value", 0).build();
        let goal = State::new().set("value", "string").build(); // Type mismatch

        let result = planner.heuristic(&current, &goal);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlannerError::IncompatibleStateTypes(msg) => {
                assert!(msg.contains("Cannot calculate distance for variable 'value'"));
            }
            _ => panic!("Expected IncompatibleStateTypes error"),
        }
    }
}
