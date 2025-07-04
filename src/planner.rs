use crate::actions::Action;
use crate::goals::Goal;
use crate::state::State;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PlannerError {
    NoPlanFound,
}

impl fmt::Display for PlannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlannerError::NoPlanFound => write!(f, "No plan found"),
        }
    }
}

impl Error for PlannerError {}

#[derive(Debug)]
pub struct Plan {
    pub actions: Vec<Action>,
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

pub struct Planner {}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner {
    pub fn new() -> Self {
        Planner {}
    }

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
        let initial_h = self.heuristic(&initial_state, &goal.desired_state);

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

            let current_g = *g_score.get(&current).unwrap();
            let transitions = self.get_valid_transitions(&current, actions);

            for (next_state, cost, action) in transitions {
                let tentative_g = current_g + cost;
                let next_h = self.heuristic(&next_state, &goal.desired_state);
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

    fn heuristic(&self, current: &State, goal: &State) -> f64 {
        let mut total_distance = 0;

        // Calculate distance for each goal requirement
        for (key, goal_val) in &goal.vars {
            match current.vars.get(key) {
                Some(current_val) => {
                    let distance = current_val.distance(goal_val);
                    total_distance += distance;
                }
                None => {
                    total_distance += 1; // Penalty for missing keys
                }
            }
        }

        total_distance as f64
    }

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

// Node wrapper for the priority queue
#[derive(Clone)]
struct NodeWrapper<N> {
    node: N,
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
        self.f_score.partial_cmp(&other.f_score).unwrap().reverse()
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

        let current = State::builder().int("value", 0).bool("flag", false).build();

        let goal = State::builder().int("value", 10).bool("flag", true).build();

        let h = planner.heuristic(&current, &goal);
        assert!(h > 0.0); // Should have some distance to goal
    }
}
