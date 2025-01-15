use crate::actions::Action;
use crate::goals::Goal;
use crate::state::WorldState;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

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
        initial_state: WorldState,
        goal: &Goal,
        actions: &[Action],
    ) -> Option<(Vec<Action>, f64)> {
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
                return Some(self.reconstruct_path(&came_from, &action_taken, &current));
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

        None
    }

    fn get_valid_transitions(
        &self,
        state: &WorldState,
        actions: &[Action],
    ) -> Vec<(WorldState, f64, Action)> {
        let mut transitions = Vec::new();
        for action in actions {
            if action.can_execute(state) {
                let new_state = action.apply_effect(state);
                transitions.push((new_state, action.cost, action.clone()));
            }
        }
        transitions
    }

    fn heuristic(&self, current: &WorldState, goal: &WorldState) -> f64 {
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
        came_from: &HashMap<WorldState, WorldState>,
        action_taken: &HashMap<WorldState, Action>,
        current: &WorldState,
    ) -> (Vec<Action>, f64) {
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
        (actions, total_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{StateOperation, StateVar};

    #[test]
    fn test_node_wrapper_ordering() {
        let state1 = WorldState::new();
        let state2 = WorldState::new();

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
    fn test_planner_simple_plan() {
        let planner = Planner::new();

        // Initial state
        let mut initial_state = WorldState::new();
        initial_state.set("has_wood", StateVar::Bool(false));

        // Goal state
        let mut goal_state = WorldState::new();
        goal_state.set("has_wood", StateVar::Bool(true));
        let goal = Goal::new("get_wood", goal_state);

        // Action to get wood
        let mut effects = HashMap::new();
        effects.insert(
            "has_wood".to_string(),
            StateOperation::Set(StateVar::Bool(true)),
        );
        let action = Action::new("get_wood", 1.0, WorldState::new(), effects);
        let actions = vec![action];

        // Find plan
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_some());

        let (actions, cost) = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name, "get_wood");
        assert_eq!(cost, 1.0);
    }

    #[test]
    fn test_planner_no_solution() {
        let planner = Planner::new();

        // Initial state
        let mut initial_state = WorldState::new();
        initial_state.set("has_wood", StateVar::Bool(false));

        // Goal state requiring something impossible
        let mut goal_state = WorldState::new();
        goal_state.set("has_gold", StateVar::Bool(true));
        let goal = Goal::new("get_gold", goal_state);

        // Action that doesn't help achieve goal
        let mut effects = HashMap::new();
        effects.insert(
            "has_wood".to_string(),
            StateOperation::Set(StateVar::Bool(true)),
        );
        let action = Action::new("get_wood", 1.0, WorldState::new(), effects);
        let actions = vec![action];

        // Should find no solution
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_none());
    }

    #[test]
    fn test_planner_multi_step_plan() {
        let planner = Planner::new();

        // Initial state
        let mut initial_state = WorldState::new();
        initial_state.set("has_wood", StateVar::Bool(false));
        initial_state.set("has_planks", StateVar::Bool(false));

        // Goal state
        let mut goal_state = WorldState::new();
        goal_state.set("has_planks", StateVar::Bool(true));
        let goal = Goal::new("get_planks", goal_state);

        // Action to get wood
        let mut wood_effects = HashMap::new();
        wood_effects.insert(
            "has_wood".to_string(),
            StateOperation::Set(StateVar::Bool(true)),
        );
        let get_wood = Action::new("get_wood", 1.0, WorldState::new(), wood_effects);

        // Action to craft planks
        let mut planks_effects = HashMap::new();
        planks_effects.insert(
            "has_planks".to_string(),
            StateOperation::Set(StateVar::Bool(true)),
        );
        let mut planks_conditions = WorldState::new();
        planks_conditions.set("has_wood", StateVar::Bool(true));
        let craft_planks = Action::new("craft_planks", 2.0, planks_conditions, planks_effects);

        let actions = vec![get_wood, craft_planks];

        // Find plan
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_some());

        let (actions, cost) = result.unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].name, "get_wood");
        assert_eq!(actions[1].name, "craft_planks");
        assert_eq!(cost, 3.0); // Total cost should be 1.0 + 2.0
    }

    #[test]
    fn test_heuristic() {
        let planner = Planner::new();

        let mut current = WorldState::new();
        current.set("value", StateVar::I64(0));
        current.set("flag", StateVar::Bool(false));

        let mut goal = WorldState::new();
        goal.set("value", StateVar::I64(10));
        goal.set("flag", StateVar::Bool(true));

        let h = planner.heuristic(&current, &goal);
        assert!(h > 0.0); // Should have some distance to goal
    }
}
