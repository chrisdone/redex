/*

Example:

$ cargo watch -x run > /dev/null
Finished dev [unoptimized + debuginfo] target(s) in 0.00s
Running `target/debug/redex`

Memory use a constant 1.7m, cpu = 100%

13873 chris     20   0   17.0m   1.8m   1.6m R 100.0  0.0   0:26.34 redex

That's good!

 */

use std::collections::HashMap;
use std::result::Result;

// Implements a trivial l-calc interpreter.
//
// Duet has cases and type classes (not necessary for such a
// demonstration) and top-level bindings (needed for recursion) and a
// renamer, which are necessary for a complete demonstration.
//
// To test this properly, I could quickly generate values of
// Expression and compile some Rust off the cuff to see whether the
// output make sense. Later, I could use the FFI with
// e.g. (https://github.com/mgattozzi/curryrs) or the modern
// equivalent.

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
struct Name(pub u64);

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
enum RenameError {
    MissingName(Name)
}

#[derive(Debug, PartialEq, Clone)]
// These boxes could be references, and the stepper could be a mutator.
enum Expression {
    Variable { name: Name },
    Constructor { name: Box<String> },
    Literal { literal: Literal },
    Application { function: Box<Expression>, argument: Box<Expression> },
    Lambda { parameter: Name, body: Box<Expression> },
    Case { scrutinee: Box<Expression>, alternatives: Vec<Box<Alternative>> }
}

#[derive(Debug, PartialEq, Clone)]
struct Alternative {
    pattern: Box<Pattern>,
    rhs: Box<Expression>
}

#[derive(Debug, PartialEq, Clone)]
enum Pattern {
    Constructor { name: Box<String>, arguments: Vec<Box<Pattern>> },
    Variable { name: Name },
    Wildcard
}

#[derive(Debug, PartialEq, Clone)]
enum Literal {
    I64Literal { i64: i64 }
}

fn main() {
    step(
        Expression::Case {
            scrutinee: Box::new(Expression::Application {
                function: Box::new(Expression::Constructor {name: Box::new("Just".to_string())}),
                argument: Box::new(Expression::Literal {
                    literal: Literal::I64Literal {i64: 123}
                })}),
            alternatives: vec![
                Box::new(Alternative {
                    pattern: Box::new(Pattern::Constructor {
                        name: Box::new("Just".to_string()),
                        arguments: vec![
                            Box::new(Pattern::Variable { name: Name(0) })
                        ]
                    }),
                    rhs: Box::new(
                        Expression::Variable { name: Name(0) }
                    )
                })
            ]
        }
    );

    // step(Expression::Variable {name: Name(0)});
    // step(Expression::Literal {literal: Literal::I64Literal {i64: 123}});
    // step(Expression::Application {
    //     function: Box::new(Expression::Lambda {
    //         parameter: Name(0),
    //         body: Box::new(Expression::Variable {name: Name(0)})
    //     }),
    //     argument: Box::new(Expression::Literal {
    //         literal: Literal::I64Literal {i64: 123}
    //     })
    // });
    // step(Expression::Application {
    //     function: Box::new(Expression::Application {
    //         function: Box::new(Expression::Lambda {
    //             parameter: Name(0),
    //             body: Box::new(Expression::Lambda {
    //                 parameter: Name(1),
    //                 body: Box::new(Expression::Application {
    //                     function: Box::new(Expression::Variable {name: Name(0)}),
    //                     argument: Box::new(Expression::Variable {name: Name(1)})})})}),
    //         argument: Box::new(Expression::Lambda {
    //             parameter: Name(2),
    //             body: Box::new(Expression::Variable {name: Name(2)})
    //         }) }),
    //     argument: Box::new(Expression::Literal {literal: Literal::I64Literal {i64: 123}})});
    // y = \f -> (\x -> f (x x)) (\x -> f (x x))

    //         \                                x ->                                                                                f
    // let x_x = Expression::Application{
    //     function: Box::new(Expression::Variable{name: Name(1)}),
    //     argument: Box::new(Expression::Variable{name: Name(1)})
    // };
    // let f_x_x = Expression::Application{
    //     function: Box::new(Expression::Variable{name: Name(0)}),
    //     argument: Box::new(x_x)
    // };
    // let lam_x_f_x_x = Expression::Lambda{
    //     parameter: Name(1),
    //     body: Box::new(f_x_x)
    // };
    // let y = Expression::Lambda{
    //     parameter: Name(0),
    //     body: Box::new(Expression::Application{
    //         function: Box::new(lam_x_f_x_x.clone()),
    //         argument: Box::new(lam_x_f_x_x)
    //     })
    // };
    // let id = Expression::Lambda{
    //     parameter: Name(0),
    //     body: Box::new(Expression::Variable{name: Name(0)})
    // };
    // step(Expression::Application{
    //     function: Box::new(y),
    //     argument: Box::new(id)
    // });
}

// Just call expand_whnf and repeat. Didn't even bother to use a loop.
fn step(e0: Expression) {
    let mut e = e0;
    loop {
        let e_clone = e.clone();
        let scope = HashMap::new();
        let mut names = 1000;
        match rename(&scope, e_clone, &mut names) {
            Err(e) => {
                println!("Rename error: {:#?}", e);
                break
            },
            Ok(e_renamed) => {
                let e_renamed_clone = e_renamed.clone();
                println!("= {:?}", e_renamed);
                let e_expanded = expand_whnf(e_renamed);
                if e_renamed_clone == e_expanded {
                    println!("Done!");
                    break
                } else {
                    e = e_expanded;
                }
            }
        }
    }
}

// Haskell equiv. https://github.com/duet-lang/duet/blob/f58e0f537c55713048fa17c723c7d0ad80a31368/src/Duet/Stepper.hs#L75
fn expand_whnf(e: Expression) -> Expression {
    match e {
        // No-ops:
        Expression::Variable{..} => e,
        Expression::Constructor{..} => e,
        Expression::Literal{..} => e,
        Expression::Lambda{..} => e,
        // Application of lambdas
        Expression::Application{function, argument} =>
            match *function {
                Expression::Lambda{parameter, body} =>
                    substitute(parameter, *body, *argument),
                Expression::Constructor{..} =>
                    Expression::Application{function, argument},
                func => {
                    let inner = expand_whnf(func);
                    Expression::Application{function: Box::new(inner), argument}
                }
            },
        Expression::Case{scrutinee, alternatives} =>
            Expression::Case{scrutinee, alternatives}
    }
}

// Haskell equiv. https://github.com/duet-lang/duet/blob/f58e0f537c55713048fa17c723c7d0ad80a31368/src/Duet/Stepper.hs#L313
fn substitute(that: Name, e: Expression, arg: Expression) -> Expression {
    match e {
        Expression::Variable { name } =>
            if name == that {
                arg
            } else {
                e
            },
        Expression::Literal{..} => e,
        Expression::Constructor{..} => e,
        Expression::Lambda{parameter, body} =>
            Expression::Lambda {
                parameter,
                body: Box::new(substitute(that, *body, arg))
            },
        Expression::Application{function, argument} =>
            Expression::Application{
                function: Box::new(substitute(that, *function, arg.clone())),
                argument: Box::new(substitute(that, *argument, arg.clone()))
            },
        Expression::Case{scrutinee, alternatives} =>
            Expression::Case {
                scrutinee,
                alternatives
            },
    }
}

// Missing

// https://github.com/duet-lang/duet/blob/f58e0f537c55713048fa17c723c7d0ad80a31368/src/Duet/Stepper.hs#L248

fn rename(scope: &HashMap<Name,Name>, e: Expression, names: &mut u64) -> Result<Expression,RenameError> {
    match e {
        Expression::Literal{..} => Ok(e),
        Expression::Constructor{..} => Ok(e),
        Expression::Application{function, argument} =>
            Ok(Expression::Application {
                function: Box::new(rename(&scope, *function, names)?),
                argument: Box::new(rename(&scope, *argument, names)?)
            }),
        Expression::Lambda{parameter, body} => {
            *names = *names + 1;
            let mut newscope = scope.clone();
            newscope.insert(parameter, Name(*names));
            Ok(Expression::Lambda {
                parameter: Name(*names),
                body: Box::new(rename(&newscope, *body, names)?)
            })
        },
        Expression::Variable{name} =>
            match scope.get(&name) {
                None => Err(RenameError::MissingName(name)),
                Some(replacement) =>
                    Ok(Expression::Variable{name: *replacement})
            },
        Expression::Case{scrutinee, alternatives} =>
            Ok(Expression::Case{
                scrutinee: Box::new(rename(scope, *scrutinee, names)?),
                alternatives
            })
    }
}
