use parser::{CommandBuilder, CommandSpec, IterParser, literal};




#[test]
fn build_and_run() {

    let cmd: CommandSpec<_,_,_,_,_> = literal("/echo").on_call(|| move |_x: usize, _y: &mut usize| {
        10 as usize
    }); 

    let x = cmd.call((6, &mut 7), "/echo");

    assert!(x.is_ok());
    assert_eq!(x.unwrap(), 10);
    let regex = cmd.parser.regex();
    println!("{}",regex);

    let x = cmd.call((6, &mut 7), "/ec");
    assert!(x.is_err())
    
    







}