#[cfg(test)]
mod tests {
    use goap::prelude::*;

    fn i(v: i64) -> StateVar {
        StateVar::I64(v)
    }

    fn f(v: f64) -> StateVar {
        StateVar::from_f64(v)
    }

    // --- Equals / NotEquals -------------------------------------------------

    #[test]
    fn equals_matches_exact_value_for_every_type() {
        assert!(Condition::Equals(i(5)).is_met_by(&i(5)));
        assert!(!Condition::Equals(i(5)).is_met_by(&i(6)));
        assert!(Condition::Equals(f(1.5)).is_met_by(&f(1.5)));
        assert!(!Condition::Equals(f(1.5)).is_met_by(&f(1.501)));
        assert!(Condition::Equals(StateVar::Bool(true)).is_met_by(&StateVar::Bool(true)));
        assert!(!Condition::Equals(StateVar::Bool(true)).is_met_by(&StateVar::Bool(false)));
        assert!(Condition::Equals("a".into_state_var()).is_met_by(&"a".into_state_var()));
        assert!(!Condition::Equals("a".into_state_var()).is_met_by(&"b".into_state_var()));
    }

    #[test]
    fn not_equals_requires_same_type() {
        assert!(not(5).is_met_by(&i(6)));
        assert!(!not(5).is_met_by(&i(5)));
        assert!(not("vault").is_met_by(&"lobby".into_state_var()));
        assert!(!not("vault").is_met_by(&"vault".into_state_var()));
        // A different type is not "not equal", it is a type error.
        assert!(!not(5).is_met_by(&f(5.0)));
        assert!(!not(5).is_met_by(&StateVar::Bool(true)));
    }

    // --- Ordered comparisons ------------------------------------------------

    #[test]
    fn at_least_and_more_than_boundaries() {
        assert!(at_least(10).is_met_by(&i(10)));
        assert!(at_least(10).is_met_by(&i(11)));
        assert!(!at_least(10).is_met_by(&i(9)));

        assert!(!more_than(10).is_met_by(&i(10)));
        assert!(more_than(10).is_met_by(&i(11)));

        assert!(at_least(2.5).is_met_by(&f(2.5)));
        assert!(!at_least(2.5).is_met_by(&f(2.499)));
        assert!(!more_than(2.5).is_met_by(&f(2.5)));
        assert!(more_than(2.5).is_met_by(&f(2.501)));
    }

    #[test]
    fn at_most_and_less_than_boundaries() {
        assert!(at_most(0).is_met_by(&i(0)));
        assert!(at_most(0).is_met_by(&i(-1)));
        assert!(!at_most(0).is_met_by(&i(1)));

        assert!(!less_than(0).is_met_by(&i(0)));
        assert!(less_than(0).is_met_by(&i(-1)));

        assert!(at_most(1.0).is_met_by(&f(1.0)));
        assert!(!at_most(1.0).is_met_by(&f(1.001)));
        assert!(!less_than(1.0).is_met_by(&f(1.0)));
        assert!(less_than(1.0).is_met_by(&f(0.999)));
    }

    #[test]
    fn between_is_inclusive_and_order_independent() {
        for c in [between(20.0, 24.0), between(24.0, 20.0)] {
            assert!(c.is_met_by(&f(20.0)));
            assert!(c.is_met_by(&f(22.0)));
            assert!(c.is_met_by(&f(24.0)));
            assert!(!c.is_met_by(&f(19.999)));
            assert!(!c.is_met_by(&f(24.001)));
        }
        assert!(between(5, 5).is_met_by(&i(5)));
        assert!(!between(5, 5).is_met_by(&i(4)));
    }

    #[test]
    fn ordered_comparisons_never_match_other_types() {
        assert!(!at_least(10).is_met_by(&f(10.0)));
        assert!(!at_least(10.0).is_met_by(&i(10)));
        assert!(!at_most(10).is_met_by(&StateVar::Bool(true)));
        assert!(!between(1, 5).is_met_by(&"3".into_state_var()));
    }

    // --- Distance -----------------------------------------------------------

    #[test]
    fn distance_is_zero_exactly_when_met() {
        let conds = [
            at_least(10),
            at_most(10),
            more_than(10),
            less_than(10),
            between(5, 10),
            10.into_condition(),
            not(10),
        ];
        for c in &conds {
            for v in -5..20 {
                assert_eq!(
                    c.is_met_by(&i(v)),
                    c.distance(&i(v)) == Some(0),
                    "{c} vs {v}"
                );
            }
        }
    }

    #[test]
    fn distance_values() {
        assert_eq!(at_least(10).distance(&i(3)), Some(7));
        assert_eq!(more_than(10).distance(&i(3)), Some(8));
        assert_eq!(at_most(10).distance(&i(15)), Some(5));
        assert_eq!(less_than(10).distance(&i(15)), Some(6));
        assert_eq!(between(5, 10).distance(&i(2)), Some(3));
        assert_eq!(between(5, 10).distance(&i(13)), Some(3));
        assert_eq!(10.into_condition().distance(&i(4)), Some(6));
        assert_eq!(10.into_condition().distance(&i(16)), Some(6));
        assert_eq!(not(10).distance(&i(10)), Some(1));

        // F64 distances are in thousandths.
        assert_eq!(at_least(2.5).distance(&f(1.5)), Some(1000));
        assert_eq!(1.002.into_condition().distance(&f(1.001)), Some(1));

        // Bool / String are 0 or 1.
        assert_eq!(
            true.into_condition().distance(&StateVar::Bool(false)),
            Some(1)
        );
        assert_eq!(
            "a".into_condition().distance(&"b".into_state_var()),
            Some(1)
        );
    }

    #[test]
    fn distance_is_none_on_type_mismatch() {
        assert_eq!(at_least(10).distance(&f(10.0)), None);
        assert_eq!(true.into_condition().distance(&i(1)), None);
        assert_eq!(not("x").distance(&StateVar::Bool(true)), None);
    }

    #[test]
    fn distance_does_not_overflow() {
        assert_eq!(
            Condition::Equals(i(i64::MAX)).distance(&i(i64::MIN)),
            Some(u64::MAX)
        );
        assert_eq!(more_than(i64::MAX).distance(&i(i64::MIN)), Some(u64::MAX));
    }

    // --- Direction ----------------------------------------------------------

    #[test]
    fn direction_points_toward_satisfaction() {
        assert_eq!(at_least(10).direction(&i(3)), Direction::Increase);
        assert_eq!(at_least(10).direction(&i(10)), Direction::Satisfied);
        assert_eq!(at_most(10).direction(&i(30)), Direction::Decrease);
        assert_eq!(10.into_condition().direction(&i(3)), Direction::Increase);
        assert_eq!(10.into_condition().direction(&i(30)), Direction::Decrease);
        assert_eq!(between(5, 10).direction(&i(1)), Direction::Increase);
        assert_eq!(between(5, 10).direction(&i(11)), Direction::Decrease);
        assert_eq!(not(10).direction(&i(10)), Direction::Either);
        assert_eq!(
            true.into_condition().direction(&StateVar::Bool(false)),
            Direction::Either
        );
    }

    // --- Conditions collection ---------------------------------------------

    #[test]
    fn conditions_all_must_hold_and_missing_fails() {
        let mut conds = Conditions::new();
        conds.set("has_key", true);
        conds.set("gold", at_least(100));

        let ok = State::builder()
            .set("has_key", true)
            .set("gold", 150)
            .build();
        assert!(conds.is_met_by(&ok));
        assert!(ok.meets(&conds));

        let poor = State::builder()
            .set("has_key", true)
            .set("gold", 50)
            .build();
        assert!(!conds.is_met_by(&poor));
        assert_eq!(conds.unmet_by(&poor), vec!["gold".to_string()]);

        let missing = State::builder().set("has_key", true).build();
        assert!(!conds.is_met_by(&missing));

        assert!(Conditions::new().is_met_by(&State::new()));
    }

    #[test]
    fn second_condition_on_same_key_replaces_first() {
        let mut conds = Conditions::new();
        conds.set("gold", at_least(100));
        conds.set("gold", at_most(10));
        assert_eq!(conds.len(), 1);
        assert_eq!(conds.get("gold"), Some(&at_most(10)));
    }

    /// Regression for BUG-002: `requires(x, 0)` used to be satisfied by any
    /// non-negative value because numbers were compared with `>=`.
    #[test]
    fn requiring_zero_means_exactly_zero() {
        let goal = Goal::builder("empty").requires("used_storage", 0).build();
        assert!(goal.is_satisfied(&State::builder().set("used_storage", 0).build()));
        assert!(!goal.is_satisfied(&State::builder().set("used_storage", 999).build()));
        assert!(!goal.is_satisfied(&State::builder().set("used_storage", -1).build()));

        let calm = Goal::builder("calm").requires("alert", at_most(0)).build();
        assert!(calm.is_satisfied(&State::builder().set("alert", 0).build()));
        assert!(calm.is_satisfied(&State::builder().set("alert", -1).build()));
        assert!(!calm.is_satisfied(&State::builder().set("alert", 1).build()));
    }

    #[test]
    fn enum_values_work_as_equality_conditions() {
        #[derive(Debug)]
        enum Weather {
            Sunny,
            Rainy,
        }
        impl std::fmt::Display for Weather {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self:?}")
            }
        }
        impl std::str::FromStr for Weather {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, ()> {
                match s {
                    "Sunny" => Ok(Weather::Sunny),
                    "Rainy" => Ok(Weather::Rainy),
                    _ => Err(()),
                }
            }
        }
        impl EnumStateVar for Weather {}

        let goal = Goal::builder("picnic")
            .requires("weather", Weather::Sunny)
            .build();
        assert!(goal.is_satisfied(&State::builder().set("weather", Weather::Sunny).build()));
        assert!(!goal.is_satisfied(&State::builder().set("weather", Weather::Rainy).build()));

        let not_rainy = Goal::builder("dry")
            .requires("weather", not(Weather::Rainy))
            .build();
        assert!(not_rainy.is_satisfied(&State::builder().set("weather", Weather::Sunny).build()));
    }

    /// BUG-015: the map is private; the accessors cover every operation.
    #[test]
    fn test_conditions_accessors() {
        let mut conditions = Conditions::new();
        assert!(conditions.is_empty());
        assert!(!conditions.contains("gold"));

        conditions.set("gold", at_least(10));
        conditions.set("flag", true);
        assert_eq!(conditions.len(), 2);
        assert!(conditions.contains("gold"));
        assert_eq!(conditions.get("gold"), Some(&at_least(10)));

        let mut keys: Vec<&String> = conditions.iter().map(|(k, _)| k).collect();
        keys.sort();
        assert_eq!(keys, ["flag", "gold"]);

        assert_eq!(conditions.remove("gold"), Some(at_least(10)));
        assert!(!conditions.contains("gold"));
        assert_eq!(conditions.len(), 1);
    }

    // Plan 005 (BUG-025, BUG-026)

    /// `Conditions::new()` is the only empty constructor.
    #[test]
    fn test_conditions_new_is_empty() {
        assert!(Conditions::new().is_empty());
        assert_eq!(Conditions::new(), Conditions::default());
    }

    /// BUG-025: the ordered helpers accept a `StateVar`, which is the
    /// fallible route for computed floats.
    #[test]
    fn test_condition_helpers_accept_state_var() {
        let value = StateVar::try_from_f64(2.5).unwrap();
        assert_eq!(at_least(value.clone()), at_least(2.5));
        assert_eq!(at_most(value.clone()), at_most(2.5));
        assert_eq!(more_than(value.clone()), more_than(2.5));
        assert_eq!(less_than(value.clone()), less_than(2.5));
        assert_eq!(
            between(value.clone(), StateVar::F64(5000)),
            between(2.5, 5.0)
        );
        assert_eq!(not(value.clone()), not(2.5));
        assert_eq!(value.into_condition(), 2.5.into_condition());
        assert!(StateVar::try_from_f64(f64::NAN).is_err());
    }

    /// BUG-025: shape errors that the helpers cannot prevent for `StateVar`
    /// arguments (or hand-built conditions) are detectable.
    #[test]
    fn test_condition_is_well_formed() {
        assert!(at_least(5).is_well_formed());
        assert!(between(1.0, 2.0).is_well_formed());
        assert!(true.into_condition().is_well_formed());
        assert!(not("x").is_well_formed());
        assert!(Condition::Equals(StateVar::Bool(true)).is_well_formed());

        assert!(!at_least(StateVar::Bool(true)).is_well_formed());
        assert!(!at_most(StateVar::from("a")).is_well_formed());
        assert!(!more_than(StateVar::Bool(false)).is_well_formed());
        assert!(!less_than(StateVar::from("z")).is_well_formed());
        assert!(!Condition::Between(StateVar::I64(1), StateVar::F64(5000)).is_well_formed());
        assert!(!Condition::Between(StateVar::Bool(false), StateVar::Bool(true)).is_well_formed());
        assert!(Condition::Between(StateVar::I64(1), StateVar::I64(5)).is_well_formed());
    }
}
