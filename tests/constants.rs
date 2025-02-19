use rhai::{Engine, EvalAltResult, ParseErrorType, Scope, INT};

#[test]
fn test_constant() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("const x = 123; x")?, 123);

    assert!(matches!(
        *engine
            .eval::<INT>("const x = 123; x = 42;")
            .expect_err("expects error"),
        EvalAltResult::ErrorParsing(ParseErrorType::AssignmentToConstant(x), _) if x == "x"
    ));

    #[cfg(not(feature = "no_index"))]
    assert!(matches!(
        *engine.consume("const x = [1, 2, 3, 4, 5]; x[2] = 42;").expect_err("expects error"),
        EvalAltResult::ErrorParsing(ParseErrorType::AssignmentToConstant(x), _) if x == "x"
    ));

    Ok(())
}

#[test]
fn test_constant_scope() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    let mut scope = Scope::new();
    scope.push_constant("x", 42 as INT);

    assert!(matches!(
        *engine.consume_with_scope(&mut scope, "x = 1").expect_err("expects error"),
        EvalAltResult::ErrorAssignmentToConstant(x, _) if x == "x"
    ));

    Ok(())
}

#[cfg(not(feature = "no_object"))]
#[test]
fn test_constant_mut() -> Result<(), Box<EvalAltResult>> {
    #[derive(Debug, Clone)]
    struct TestStruct(INT); // custom type

    let mut engine = Engine::new();

    engine
        .register_type_with_name::<TestStruct>("TestStruct")
        .register_get("value", |obj: &mut TestStruct| obj.0)
        .register_fn("update_value", |obj: &mut TestStruct, value: INT| {
            obj.0 = value
        });

    let mut scope = Scope::new();

    scope.push_constant("MY_NUMBER", TestStruct(123));

    assert_eq!(
        engine.eval_with_scope::<INT>(
            &mut scope,
            r"
                MY_NUMBER.update_value(42);
                MY_NUMBER.value
            ",
        )?,
        42
    );

    Ok(())
}
