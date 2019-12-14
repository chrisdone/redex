#[derive(Debug)]
enum Expression<'a> {
    VariableExpression { name: &'a str },
    LiteralExpression { literal: Literal },
    ApplicationExpression { function: Box<Expression<'a>>, argument: Box<Expression<'a>> },
    LambdaExpression { variable: &'a str, body: Box<Expression<'a>> }
}

#[derive(Debug)]
enum Literal {
    IntegerLiteral { i64: i64 }
}

fn main() {
    step(Expression::VariableExpression {name: "x"});
    step(Expression::LiteralExpression {literal: Literal::IntegerLiteral {i64: 123}});
    step(Expression::ApplicationExpression {
        function: Box::new(Expression::LambdaExpression {
            variable: "x",
            body: Box::new(Expression::VariableExpression {name: "x"})
        }),
        argument: Box::new(Expression::LiteralExpression {
            literal: Literal::IntegerLiteral {i64: 123}
        })
    });


}

fn step(e: Expression) {
    println!("before = {:#?}", e);
    let e_expanded = expand_whnf(e);
    println!("after = {:#?}", e_expanded);
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
                Expression::LambdaExpression{variable, body} =>
                    substitute(variable, *body, *argument),
                func =>
                    Expression::ApplicationExpression{function: Box::new(func), argument}
            }
    }
}

fn substitute<'a>(_name: &'a str, _body: Expression<'a>, arg: Expression<'a>) -> Expression<'a> {
    arg
}
