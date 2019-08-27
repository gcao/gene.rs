#[macro_use]
extern crate gene;

use std::collections::HashMap;

use ordered_float::OrderedFloat;

use gene::compiler2::Compiler;
use gene::parser::Parser;
use gene::types::Value;
use gene::vm::VirtualMachine;

#[test]
fn test_basic_stmts() {
    {
        let mut parser = Parser::new("null");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Null);
    }
    {
        let mut parser = Parser::new("1");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("1.1");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Float(OrderedFloat(1.1)));
    }
    {
        let mut parser = Parser::new("true");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Boolean(true));
    }
    {
        let mut parser = Parser::new("\"ab\"");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::String("ab".to_string()));
    }
    {
        let mut parser = Parser::new("[]");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(Vec::new()));
    }
    {
        let mut parser = Parser::new("[1]");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![Value::Integer(1)]));
    }
    {
        let mut parser = Parser::new("[1]");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![Value::Integer(1)]));
    }
    {
        let mut parser = Parser::new("{}");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Map(HashMap::new()));
    }
    {
        let mut parser = Parser::new("{^key 1}");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(
            *result,
            Value::Map(map! {
                "key" => Value::Integer(1),
            })
        );
    }
}

#[test]
fn test_variables() {
    {
        let mut parser = Parser::new("
            # Define variable <a>
            (var a 1)
            # Return <a>'s value
            a
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("
            (var a 1)
            (var b 3)
            [a 2 b]
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
    }
    {
        let mut parser = Parser::new("
            (var a 1)
            (var b 2)
            [a b (a + b)]
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
    }
    {
        let mut parser = Parser::new("
            (var a 1)
            (var b 2)
            {^ka a ^kb b}
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(
            *result,
            Value::Map(map! {
                "ka" => Value::Integer(1),
                "kb" => Value::Integer(2),
            })
        );
    }
}

#[test]
fn test_binary_operations() {
    {
        let mut parser = Parser::new("
            (1 + 2)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(3));
    }
    {
        let mut parser = Parser::new("
            (1 < 2)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Boolean(true));
    }
}

#[test]
fn test_assignments() {
    {
        let mut parser = Parser::new("
            (var a 1)
            (a = 2)
            a
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(2));
    }
}

#[test]
fn test_ifs() {
    {
        let mut parser = Parser::new("
            (if true 1 else 2)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
}

#[test]
fn test_functions() {
    {
        let mut parser = Parser::new("
            (fn f _ 1)
            (f)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("
            (fn f a a)
            (f 1)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("
            (fn f [a b] (a + b))
            (f 1 2)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(3));
    }
    {
        let mut parser = Parser::new("
            (fn f _ 1)
            (fn g _ (f))
            (g)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("
            (fn f [a b] (a + b))
            (fn g [c d] (f c d))
            (g 1 2)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(3));
    }
}

#[test]
fn test_fibonacci() {
    {
        let mut parser = Parser::new("
            (fn fibonacci n
                (if (n < 2)
                    n
                else
                    ((fibonacci (n - 1)) + (fibonacci (n - 2)))
                )
            )
            (fibonacci 6)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(8));
    }
}
