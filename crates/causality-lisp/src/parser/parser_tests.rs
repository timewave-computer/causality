#[cfg(test)]
mod tests {
    use crate::parser::{parse, parse_first, parse_program_str};
    use crate::parser::error::ParseError;
    use causality_types::expr::ast::{Atom, AtomicCombinator, Expr};
    use causality_types::primitive::string::Str;
    use causality_types::primitive::number::Number;

    #[test]
    fn test_parse_atoms() {
        // Test parsing integer
        let result = parse("42").expect("Failed to parse integer");
        match result {
            Expr::Atom(Atom::Integer(i)) => assert_eq!(i, 42),
            _ => panic!("Expected Integer atom"),
        }
        
        // Test parsing string
        let result = parse("\"hello world\"").expect("Failed to parse string");
        match result {
            Expr::Atom(Atom::String(s)) => assert_eq!(s.as_str(), "hello world"),
            _ => panic!("Expected String atom"),
        }
        
        // Test parsing boolean
        let result = parse("true").expect("Failed to parse boolean");
        match result {
            Expr::Atom(Atom::Boolean(b)) => assert_eq!(b, true),
            _ => panic!("Expected Boolean atom"),
        }
        
        // Test parsing nil
        let result = parse("nil").expect("Failed to parse nil");
        match result {
            Expr::Atom(Atom::Nil) => {},
            _ => panic!("Expected Nil atom"),
        }
    }

    #[test]
    fn test_parse_variables() {
        // Test parsing variables/symbols
        let result = parse("my-variable").expect("Failed to parse variable");
        match result {
            Expr::Var(s) => assert_eq!(s.as_str(), "my-variable"),
            _ => panic!("Expected Var expression"),
        }
        
        // Test parsing special symbols like + or -
        let result = parse("+").expect("Failed to parse + symbol");
        match result {
            Expr::Var(s) => assert_eq!(s.as_str(), "+"),
            _ => panic!("Expected Var expression for +"),
        }
    }

    #[test]
    fn test_parse_applications() {
        // Test parsing function application (+ 1 2)
        let result = parse("(+ 1 2)").expect("Failed to parse application");
        
        match result {
            Expr::Apply(func, args) => {
                match *func {
                    Expr::Var(s) => assert_eq!(s.as_str(), "+"),
                    _ => panic!("Expected Var as function"),
                }
                
                assert_eq!(args.len(), 2);
                
                match &args[0] {
                    Expr::Atom(Atom::Integer(i)) => assert_eq!(*i, 1),
                    _ => panic!("Expected Integer for first arg"),
                }
                
                match &args[1] {
                    Expr::Atom(Atom::Integer(i)) => assert_eq!(*i, 2),
                    _ => panic!("Expected Integer for second arg"),
                }
            },
            _ => panic!("Expected Apply expression"),
        }
        
        // Test parsing nested application
        let result = parse("(+ (* 2 3) 4)").expect("Failed to parse nested application");
        
        match result {
            Expr::Apply(func, args) => {
                match *func {
                    Expr::Var(s) => assert_eq!(s.as_str(), "+"),
                    _ => panic!("Expected Var as function"),
                }
                
                assert_eq!(args.len(), 2);
                
                // Check the nested expression (* 2 3)
                match &args[0] {
                    Expr::Apply(nested_func, nested_args) => {
                        match **nested_func {
                            Expr::Var(ref s) => assert_eq!(s.as_str(), "*"),
                            _ => panic!("Expected Var as nested function"),
                        }
                        
                        assert_eq!(nested_args.len(), 2);
                        
                        match &nested_args[0] {
                            Expr::Atom(Atom::Integer(i)) => assert_eq!(*i, 2),
                            _ => panic!("Expected Integer for first nested arg"),
                        }
                        
                        match &nested_args[1] {
                            Expr::Atom(Atom::Integer(i)) => assert_eq!(*i, 3),
                            _ => panic!("Expected Integer for second nested arg"),
                        }
                    },
                    _ => panic!("Expected nested Apply expression"),
                }
                
                match &args[1] {
                    Expr::Atom(Atom::Integer(i)) => assert_eq!(*i, 4),
                    _ => panic!("Expected Integer for second arg"),
                }
            },
            _ => panic!("Expected Apply expression"),
        }
    }

    #[test]
    fn test_parse_lambda() {
        // Test parsing lambda expression (fn (x y) (+ x y))
        let result = parse("(fn (x y) (+ x y))").expect("Failed to parse lambda");
        
        match result {
            Expr::Lambda(params, body) => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].as_str(), "x");
                assert_eq!(params[1].as_str(), "y");
                
                match *body {
                    Expr::Apply(func, args) => {
                        match *func {
                            Expr::Var(s) => assert_eq!(s.as_str(), "+"),
                            _ => panic!("Expected Var as function in lambda body"),
                        }
                        
                        assert_eq!(args.len(), 2);
                        
                        match &args[0] {
                            Expr::Var(s) => assert_eq!(s.as_str(), "x"),
                            _ => panic!("Expected Var for first arg in lambda body"),
                        }
                        
                        match &args[1] {
                            Expr::Var(s) => assert_eq!(s.as_str(), "y"),
                            _ => panic!("Expected Var for second arg in lambda body"),
                        }
                    },
                    _ => panic!("Expected Apply expression in lambda body"),
                }
            },
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_parse_combinators() {
        // Test parsing atomic combinators like if
        let result = parse("(if true 1 2)").expect("Failed to parse if combinator");
        
        match result {
            Expr::Apply(func, args) => {
                match *func {
                    Expr::Var(s) => assert_eq!(s.as_str(), "if"),
                    _ => panic!("Expected Var for if combinator"),
                }
                
                assert_eq!(args.len(), 3);
                
                match &args[0] {
                    Expr::Atom(Atom::Boolean(b)) => assert_eq!(*b, true),
                    _ => panic!("Expected Boolean for condition"),
                }
                
                match &args[1] {
                    Expr::Atom(Atom::Integer(i)) => assert_eq!(*i, 1),
                    _ => panic!("Expected Integer for true branch"),
                }
                
                match &args[2] {
                    Expr::Atom(Atom::Integer(i)) => assert_eq!(*i, 2),
                    _ => panic!("Expected Integer for false branch"),
                }
            },
            _ => panic!("Expected Apply expression for if combinator"),
        }
    }
} 