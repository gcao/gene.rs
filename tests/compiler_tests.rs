use gene::compilable::Compiler;
use gene::parser::Parser;
use gene::types::Value;

#[test]
fn test_basic_stmts() {
    let mut compiler = Compiler::new();
    {
        let mut parser = Parser::new("1");
        let parsed = parser.parse();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let block = module.get_default_block();
        // assert_eq!(block.instructions, vec![
        // ]);
    }
}
