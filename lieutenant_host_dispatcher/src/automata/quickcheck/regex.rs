use core::panic;

use rand::{prelude::ThreadRng, Rng};
use regex_generate::Generator;
use regex_syntax::Parser;

use crate::automata::{regex::we_suport_regex, NFA};

const CHARSET: [char; 71] = [
    'a', 'b', 'c', 'd', 'z', // 2byte
    'ϱ', 'ϯ', 'Ϯ', 'ϯ', 'ϰ', 'ΰ', 'ί', 'ή', 'έ', 'Ͱ', 'ͱ', 'Ͳ', 'Ͱ', //'ඌ', //3 byte
    // 'ଵ', 'ଶ', 'ଷ', 'ସ', '୵', '୶', '୷', 'வ', 'ஶ', 'ஷ', 'ᬵ', '㬵', '㭵', '㮵', '㯵', '㬶', '㬷',
    // '䬷', //4 byte
    // '🚈', '🚉', '🙉', '🙇', '😌', '򟘈', '󟘈', '󜘈', '󄖄', '}', '~', // edgecase 2byte
    // 'À', 'Á', 'Â', 'Ã', 'ß', 'Þ', 'Ý', '߿', '߾', '޿', // edgecas      e 3byte
    'က', 'ခ', 'ဂ', '၀', 'ႀ', // edgecase 4 byte
    '񀀀', '𿿿', '𿿾', '𿾿', '𾿿', '󿿿', '*', '(', ')', '*', '(', ')', '*', '(', ')', '*', '(', ')',
    '*', '(', ')', '*', '(', ')', '*', '(', ')', '*', '(', ')', '*', '(', ')',
    '\\', '\\', '\\', '\\', '\\', '\\', '\\', '\\', 
    '?', '?', '?', '+', ':', '[',']'
];

const REGEX_LEN: usize = 20;
const NUM_GENERATED_MATCHES: usize = 100;
const MAX_REPEAT: u32 = 6;

fn random_regex(rng: &mut ThreadRng) -> String {
    loop {
        let regex: String = (0..REGEX_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        let hir = Parser::new().parse(&regex);
        match hir {
            Ok(x) => {
                if crate::automata::regex::we_suport_hir(&x).is_ok() {
                    return regex;
                }
                continue;
            }
            Err(_e) => continue,
        }
    }
}

// n is the length of the resulting vector.
fn random_matches<R: Rng>(regex: &str, rng: &mut R, n: usize) -> Vec<String> {
    let mut gen = Generator::new(&regex, rng, MAX_REPEAT).unwrap();
    let mut buffer = vec![];
    let buffer_ptr = &mut buffer;

    let mut result = Vec::with_capacity(n);

    for _ in 0..n {
        buffer_ptr.clear();
        gen.generate(buffer_ptr).unwrap();
        let example = String::from_utf8(buffer_ptr.to_vec()).unwrap();
        result.push(example)
    }

    result
}

fn mutate<R: Rng>(case: &str, rng: &mut R, n: usize) -> Vec<String> {
    let mut result = Vec::with_capacity(n);
    let refference_bytes = case.clone().as_bytes().to_owned();

    while result.len() < n  && n > 0 && ! case.is_empty() {
        let mut bytes = refference_bytes.clone();

        // Choose a random byte and change it by 1
        let index: usize = rng.gen_range(0..bytes.len());
        let up: bool = rng.gen_bool(0.5);

        if up {
            bytes[index] += 1;
        } else {
            bytes[index] -= 1;
        }

        if let Ok(variant) = std::str::from_utf8(&bytes) {
            result.push(variant.to_owned())
        }
    }

    result
}

#[test]
#[ignore = "For fuzzing"]
fn fuzzing1() {
    let mut rounds: u64 = 0;
    loop {
        let mut rng = rand::thread_rng();
        let regex = random_regex(&mut rng);

        assert!(we_suport_regex(regex.as_str()).is_ok());

        let mut gen = Generator::new(&regex, rng, MAX_REPEAT).unwrap();
        let mut buffer = vec![];
        let buffer_ptr = &mut buffer;

        let nfa = NFA::regex(&regex).expect(&format!("Failed to build nfa for {}", &regex));
        for _ in 0..NUM_GENERATED_MATCHES {
            buffer_ptr.clear();
            gen.generate(buffer_ptr).unwrap();
            let example = String::from_utf8(buffer_ptr.to_vec()).unwrap();
            println!("example: {}, regex: {}", example, regex);
            if !nfa.find(&example).is_ok() {
                panic!(
                    "Error: The nfa failed to match {} when the regex was {}. ",
                    example, regex
                );
            }
        }

        rounds += 1;

        println!("Number completed: {}", rounds);
    }
}

#[test]
#[ignore = "For fuzzing"]
fn fuzzing_2() {
    /*
        1) Generate a random regex string
        2) Generate matches for it.
        3) Modify the match at the byte level, and see if
        // a) The modification does not brake utf-8
        // b) That is the regex crate matches it, then we also do it.
        //    assuming the regex does not use any features we dont suport.
    */
    let mut rng = rand::thread_rng();
    let mut loop_count : u64 = 0;

    loop {
        let regex = random_regex(&mut rng);
        let matches = random_matches(&regex, &mut rng, 10);


        let regex_crate_regex = regex::Regex::new(&format!("^{}$",&regex)).expect(&format!(
            "We accepted a string: {} as valid regex that the regex crate does not.",
            &regex
        ));

        let nfa = NFA::regex(&regex).unwrap();
        

        for m in matches {
            let mut modifications = mutate(&m, &mut rng, 1000);
            let mut keepers = Vec::new();
            let mut rounds = 0;
            loop {

                keepers.clear();
                rounds += 1;

                if modifications.len() == 0 {
                    break;
                }

                for change in modifications.iter() {
                    let we_match = nfa.find(&change).is_ok();
                    let they_match = regex_crate_regex.is_match(change);

                    if we_match == true &&  they_match == false {
                        panic!("We matched, but the regex crate does not, for the regex: {}, on the input: {}",&regex,&change);
                    }

                    if we_match == false &&  they_match == true {
                        panic!("The regex crate matched, but we do not, for the regex: {}, on the input: {}",&regex,&change);
                    }

                    if we_match == true {
                        keepers.push(change.clone());
                    }
                }

                modifications.clear();
                modifications.extend(keepers.clone());

                if rounds > 1000 {
                    break;
                }
            }
            loop_count += 1;
            println!("One done {}",loop_count)

        }
    }
}

const CASES:[(&'static str, &'static str, bool);10] = [
    (r"num: (?:[0-9]|[1-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])", "num: 255", true),
    (r"num: (?:[0-9]|[1-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])","num: 256",false),
    (r"[0-9]*\.?[0-9]+","0.1",true),
    (r"[-+]?[0-9]*\.?[0-9]+","0.1",true),
    (r"[-+]+[0-9]*\.?[0-9]+","+0.1",true),
    (r"[-+]+[0-9]*\.?[0-9]+","++0.1",true),
    (r"[-+]+[0-9]*\.?[0-9]+","+-0.1",true),
    (r"(?i-u)[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,4}","my.email@gmail.COM",true),
    (r"(?i-u)[A-Z\.]+","my.email",true),
    (r"(?-u)(19|20)\d\d[- /.](0[1-9]|1[012])[- /.](0[1-9]|[12][0-9]|3[01])","1900-01-01",true),
];

#[test]
fn selective_testcases_regex() {
    for (_id, (regex, input, should_match))  in CASES.iter().enumerate() {
        
        // Certain regex features we don't suport and we want those to not be part of the testcases. 
        // thats why i check for it here, 
        we_suport_regex(regex).unwrap();

        let nfa = NFA::regex(regex).unwrap();
        let re = regex::Regex::new(&format!("^{}$",regex)).unwrap();
        let we_match = nfa.find(input).is_ok();
        let they_match = re.is_match(input);

        // println!("-----------------------");
        //println!("they match: {}",they_match);
        //println!("we match: {}", we_match);
        //println!("{}",nfa);
        // println!("{:?}",nfa);
        //println!("{:?}",)
        assert_eq!(they_match,*should_match);
        assert!(we_match == they_match);
    }

}

