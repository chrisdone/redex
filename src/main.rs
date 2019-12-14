#[derive(Debug, PartialEq, Clone)]
enum Expression<'a> {
    VariableExpression { name: &'a str },
    LiteralExpression { literal: Literal },
    ApplicationExpression { function: Box<Expression<'a>>, argument: Box<Expression<'a>> },
    LambdaExpression { parameter: &'a str, body: Box<Expression<'a>> }
}

#[derive(Debug, PartialEq, Clone)]
enum Literal {
    IntegerLiteral { i64: i64 }
}

fn main() {
    // step(Expression::VariableExpression {name: "x"});
    // step(Expression::LiteralExpression {literal: Literal::IntegerLiteral {i64: 123}});
    // step(Expression::ApplicationExpression {
    //     function: Box::new(Expression::LambdaExpression {
    //         parameter: "x",
    //         body: Box::new(Expression::VariableExpression {name: "x"})
    //     }),
    //     argument: Box::new(Expression::LiteralExpression {
    //         literal: Literal::IntegerLiteral {i64: 123}
    //     })
    // });
    step(Expression::ApplicationExpression {
        function: Box::new(Expression::ApplicationExpression {
            function: Box::new(Expression::LambdaExpression {
                parameter: "x",
                body: Box::new(Expression::LambdaExpression {
                    parameter: "y",
                    body: Box::new(Expression::ApplicationExpression {
                        function: Box::new(Expression::VariableExpression {name: "x"}),
                        argument: Box::new(Expression::VariableExpression {name: "y"})})})}),
            argument: Box::new(Expression::LambdaExpression {
                parameter: "z",
                body: Box::new(Expression::VariableExpression {name: "z"})
            }) }),
    argument: Box::new(Expression::LiteralExpression {literal: Literal::IntegerLiteral {i64: 123}})})
}

fn step(e: Expression) {
    let e_clone = e.clone();
    println!("= {:#?}", e);
    let e_expanded = expand_whnf(e);
    if e_clone == e_expanded {
        println!("Done!")
    } else {
        step(e_expanded);
    }
}

fn expand_whnf(e: Expression) -> Expression {
    match e {
        // No-ops:
        Expression::VariableExpression{..} => e,
        Expression::LiteralExpression{..} => e,
        Expression::LambdaExpression{..} => e,
        // Application of lambdas
        Expression::ApplicationExpression{function, argument} =>
            match *function {
                Expression::LambdaExpression{parameter, body} =>
                    substitute(parameter, *body, *argument),
                func => {
                    let inner = expand_whnf(func);
                    Expression::ApplicationExpression{function: Box::new(inner), argument}
                }
            }
    }
}

fn substitute<'a>(that: &'a str, e: Expression<'a>, arg: Expression<'a>) -> Expression<'a> {
    match e {
        Expression::VariableExpression { name } =>
            if name == that {
                arg
            } else {
                e
            },
        Expression::LiteralExpression{..} => e,
        Expression::LambdaExpression{parameter, body} =>
            if parameter == that {
                Expression::LambdaExpression{parameter, body}
            } else {
                Expression::LambdaExpression {
                    parameter,
                    body: Box::new(substitute(that, *body, arg))
                }
            },
        Expression::ApplicationExpression{function, argument} =>
            Expression::ApplicationExpression{
                function: Box::new(substitute(that, *function, arg.clone())),
                argument: Box::new(substitute(that, *argument, arg.clone()))
            }
    }
}
