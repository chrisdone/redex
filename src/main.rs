#[derive(Debug)]
enum Expression {
  VariableExpression { name: String },
  LiteralExpression { literal: Literal },
  ApplicationExpression { function: Box<Expression>, argument: Box<Expression> },
  LambdaExpression { variable: String, body: Box<Expression> }
}

#[derive(Debug)]
enum Literal {
  IntegerLiteral { i64: i64 }
}

fn main() {
    let demo = Expression::VariableExpression {name: "x".to_string()};
    println!("demo = {:#?}", demo);
}
