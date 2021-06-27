use core::panic;

use regex_syntax::hir::ClassUnicodeRange;

use crate::automata::{AutomataBuildError,NFA, state::StateId};

/*

Tabele for utf-8 byte layout. We need to generate a nfa that recognises a range.
Its sort of like creating a regular expression for a number between n and m.
You think its easy, untill you realise there are a lot of edgecases.

+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+
| 1st Byte  | 2nd Byte  | 3rd Byte  | 4th Byte  | Number of Free Bits  | Maximum Expressible Unicode Value |
+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+
| 0xxxxxxx  |           |           |           | 7                    | 007F hex (127)                    |
| 110xxxxx  | 10xxxxxx  |           |           | (5+6)=11             | 07FF hex (2047)                   |
| 1110xxxx  | 10xxxxxx  | 10xxxxxx  |           | (4+6+6)=16           | FFFF hex (65535)                  |
| 11110xxx  | 10xxxxxx  | 10xxxxxx  | 10xxxxxx  | (3+6+6+6)=21         | 10FFFF hex (1,114,111)            |
+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+


+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+
| 1st Byte  | 2nd Byte  | 3rd Byte  | 4th Byte  | Number of Free Bits  | Maximum Expressible Unicode Value |
+-----------+-----------+-----------+-----------+----------------------+-----------------------------------+
|  0..=191  |           |           |           | 7                    | 007F hex (127)                    |
| 192..=223 | 128..=191 |           |           | (5+6)=11             | 07FF hex (2047)                   |
| 224..=239 | 128..=191 | 128..=191 |           | (4+6+6)=16           | FFFF hex (65535)                  |
| 240..=247 | 128..=191 | 128..=191 | 128..=191 | (3+6+6+6)=21         | 10FFFF hex (1,114,111)            |
+-----------+-----------+-----------+-----------+----------------------+-----------------------------------
*/


/**
    Every char in uf8 encoding can be one to four bytes long. Our NFA works on the byte level, so
    we need to construct a gadget for the non ascii chars. 
    
    This function modifies the given nfa to recognise any utf-8 char that uses n bytes.

*/
fn any_char_of_length_n(n: usize, start: StateId, end: StateId, nfa: &mut NFA) -> Result<(), AutomataBuildError>{
    
    match n {
        1 => {
            nfa.push_connections(start, end, 0..=191)?;
        },

        2 => {
            let a = nfa.push_state();
            nfa.push_connections(start, a,192..=223)?;
            nfa.push_connections(a,end,128..=191)?;
        },

        3 => {
            let a = nfa.push_state();
            let b = nfa.push_state();
            nfa.push_connections(start, a, 224..=239)?;
            nfa.push_connections(a,b,128..=191)?;
            nfa.push_connections(b,end,128..=191)?;
        }
        4 => {
            let a = nfa.push_state();   
            let b = nfa.push_state();   
            let c = nfa.push_state();

            nfa.push_connections(start, a, 240..=247)?;
            nfa.push_connections(a,b,128..=191)?;
            nfa.push_connections(b,c,128..=191)?;
            nfa.push_connections(c,end,128..=191)?;
        }

        _ => {
            panic!("any_char_of_length_n was given a n not equal to 1,2,3 or 4");
        }

    }

    Ok(())
}

/**
    Modifies the nfa such that it recognises any char that has the same utf-8 encoding length and
    comes before the given char c.
*/
fn below_or_eq(c: char, start: StateId, end: StateId, nfa: &mut NFA) -> Result<(), AutomataBuildError> {
    let mut bytes = [0u8;4];
    c.encode_utf8(&mut bytes);

    match c.len_utf8() {
        1 => {
            nfa.push_connections(start,end,0..=bytes[0])?;
        },

        2 => {
            
            // First byte is equal, so the next byte has to be equal or less
            {
                let a = nfa.push_state();
                nfa.push_connection(start,a,bytes[0])?;
                nfa.push_connections(a,end,128..=bytes[1])?;
            }
            
            // First byte is less, so the next byte can be anything
            {
                if bytes[0] != 192 {
                    // if this if check fails then the first byte can't be less
                    let a = nfa.push_state();
                    nfa.push_connections(start,a,192..bytes[0])?;
                    nfa.push_connections(a,end,128..=191)?;
                }
            }

        },

        3 => {
            // First and second byte is equal, last is less or eq,
            {
                let a = nfa.push_state();
                let b = nfa.push_state();
                nfa.push_connection(start, a,bytes[0])?;
                nfa.push_connection(a, b, bytes[1])?;
                nfa.push_connections(b, end, 128..=bytes[2])?;
            }

            // First byte is equal, second is less, last is anything
            {
                if bytes[1] != 128 {
                    // if this if check failes then the second byte cant be less.
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    nfa.push_connection(start,a,bytes[0])?;
                    nfa.push_connections(a,b,128..bytes[1])?;
                    nfa.push_connections(b, end, 128..=191)?;
                }
            }

            // First byte is less, second and third is anything.
            {
                if bytes[0] != 224 {
                    // if this check fails then the first bytee is 224, and there exists
                    // no byte less then it.
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    nfa.push_connections(start, a, 224..bytes[0])?;
                    nfa.push_connections(a,b,128..=191)?;
                    nfa.push_connections(b,end,128..=191)?;
                }
            }
        },

        4 => {
            
            // First, second and third byte is equal, last is less or eq
            {
                let a = nfa.push_state();
                let b = nfa.push_state();
                let c = nfa.push_state();

                nfa.push_connection(start,a,bytes[0])?;
                nfa.push_connection(a,b,bytes[1])?;
                nfa.push_connection(b,c,bytes[2])?;
                nfa.push_connections(c,end,128..=bytes[3])?;
            }

            // First and Second byte is equal, the third byte is less, and the fourth is anything.
            {
                if bytes[2] != 128 {
                    // If this check fails then the third byte can't be less then 128
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    let c = nfa.push_state();
                    nfa.push_connection(start,a,bytes[0])?;
                    nfa.push_connection(a,b,bytes[1])?;
                    nfa.push_connections(b,c,128..bytes[2])?;
                    nfa.push_connections(c,end,128..=191)?;
                }
            }

            // First byte is equal, second is less, and rest is anything
            {
                if bytes[1] != 128 {
                    // If this check fails, then there exists no value such that the second is
                    // less.
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    let c = nfa.push_state();
                    nfa.push_connection(start,a, bytes[0])?;
                    nfa.push_connections(a,b,128..bytes[1])?;
                    nfa.push_connections(b,c,128..=191)?;
                    nfa.push_connections(c,end,128..=191)?;
                }
            }

            // First byte is less, rest is anything
            {
                if bytes[0] != 240 {
                    // If this check fails, then there exists no value such that the first is less.
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    let c = nfa.push_state();
                    nfa.push_connections(start,a,240..bytes[0])?;
                    nfa.push_connections(a,b,128..=191)?;
                    nfa.push_connections(b,c,128..=191)?;
                    nfa.push_connections(c,end,128..=191)?;

                }
            }
        },
        
        _ => {
            panic!();
        }
    }

    Ok(())
}

/**
    Modifies the nfa such that it recognuses any char that has the same utf-8 encoding length and 
    comes after (inclusive) the given char c. 
*/
fn over_or_eq(c: char, start: StateId, end: StateId, nfa: &mut NFA) -> Result<(), AutomataBuildError> {
    let mut bytes = [0u8;4];
    c.encode_utf8(&mut bytes);

    match c.len_utf8() {

        1 => {
            nfa.push_connections(start, end, bytes[0]..=191)?;
        }, 

        2 => {
            // first is equal second is equal or greater.
            {
                let a = nfa.push_state();
                nfa.push_connection(start,a, bytes[0])?;
                nfa.push_connections(a, end, bytes[1]..=191)?;
            }

            // first is greater, second is anything
            {
                if bytes[0] != 223 {
                    // If this check fails, then no byte exists greater then it.
                    let a = nfa.push_state();
                    nfa.push_connections(start, a, (bytes[0]+1)..=223 )?;
                    nfa.push_connections(a, end, 128..=191)?;
                }
            }
        },

        3 => {
            // first and second is equal and third is eq or greater
            {
                let a = nfa.push_state();
                let b = nfa.push_state();
                nfa.push_connection(start, a, bytes[0])?;
                nfa.push_connection(a, b, bytes[1])?;
                nfa.push_connections(b, end, bytes[2]..=191)?;
            } 

            // first is equal, second is greater, third is anything
            {
                if bytes[1] != 191 {
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    nfa.push_connection(start,a, bytes[0])?;
                    nfa.push_connections(a,b, (bytes[1]+1)..191)?;
                    nfa.push_connections(b,end, 128..191)?;
                }
            }

            // first is greater, second and third is anything
            {
                if bytes[0] != 239 {
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    nfa.push_connections(start,a, (bytes[0]+1)..=239)?;
                    nfa.push_connections(a,b, 128..191)?;
                    nfa.push_connections(b,end, 128..191)?;
                }
            }

        },

        4 => {
            
            // First eq, Second eq, Third is equal, last is anything greater or eq.
            {
                let a = nfa.push_state();
                let b = nfa.push_state();
                let c = nfa.push_state();

                nfa.push_connection(start, a, bytes[0])?;
                nfa.push_connection(a, b, bytes[1])?;
                nfa.push_connection(b, c, bytes[2])?;
                nfa.push_connections(c, end, bytes[3]..=191)?;
            }

            // First eq, Second eq, Third is greater, Last is anything
            {
                if bytes[2] != 191 {
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    let c = nfa.push_state();

                    nfa.push_connection(start, a, bytes[0])?;
                    nfa.push_connection(a, b, bytes[1])?;
                    nfa.push_connections(b, c, (bytes[2]+1)..=191)?;
                    nfa.push_connections(c, end, 128..=191)?;
                }

            }

            // First eq, Second is greater, Third is anything, Last is anything.
            {   
                if bytes[1] != 191 {
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    let c = nfa.push_state();

                    nfa.push_connection(start, a, bytes[0])?;
                    nfa.push_connections(a, b, (bytes[1]+1)..=191)?;
                    nfa.push_connections(b,c, 128..=191)?;
                    nfa.push_connections(c, end, 128..=191)?;
                }   
            }

            // First is greater, Second is anything, Third is anything, last is anything.
            {
                if bytes[0] != 247 {
                    let a = nfa.push_state();
                    let b = nfa.push_state();
                    let c = nfa.push_state();

                    nfa.push_connections(start, a, (bytes[0]+1)..=247)?;
                    nfa.push_connections(a, b, 128..=191)?;
                    nfa.push_connections(b, c, 128..=191)?;
                    nfa.push_connections(c, end, 128..=191)?;
                }
            }
        },

        _ => {
            unreachable!("They changed utf-8?");
        }

    }

    Ok(())
}


fn between2(from: [u8;2], to: [u8; 2], start: StateId, end: StateId, nfa: &mut NFA) -> Result<(), AutomataBuildError> {
    
    match (from[0] == to[0], from[1] == to[1]) {
        (true, _) => {
            let a = nfa.push_state();
            nfa.push_connection(start, a, from[0])?;
            nfa.push_connections(a, end, from[1]..=to[1])?;
        },
        (false,_) => {

            // First eq to from[0]
            // Last greater then or eq from[1] (but still a valid second byte)
            {
                let a = nfa.push_state();
                nfa.push_connection(start, a, from[0])?;
                nfa.push_connections(a, end, from[1]..=191)?;
            }
            
            // First eq to[0]
            // Last is less then or eq to[1]
            {
                let a = nfa.push_state();
                nfa.push_connection(start, a, to[0])?;
                nfa.push_connections(a, end, 128..=to[1])?;

            }

            // First betwen from[0] and to[0] (if that is possible)
            // Last is any valid second byte.
            {
                if to[0] - from[0] > 1 {
                    let a = nfa.push_state();
                    nfa.push_connections(start, a, from[0]+1..to[0])?;
                    nfa.push_connections(a,end, 128..=191)?;
                }
            }
        }
    }
    
    Ok(())
}

fn between3(from: [u8;3], to: [u8; 3], start: StateId, end: StateId, nfa: &mut NFA) -> Result<(), AutomataBuildError> {
    if from[0] == to[0]{
        let a = nfa.push_state();
        nfa.push_connection(start, a, from[0])?;
        between2([from[1],from[2]], [to[1],to[2]], a, end, nfa)?;
    } else {

        // First eq from[0]
        // Second eq from[1]
        // Last eq or greater then from[2]
        {
            let a = nfa.push_state();
            let b = nfa.push_state();

            nfa.push_connection(start,a, from[0])?;
            nfa.push_connection(a,b, from[1])?;
            nfa.push_connections(b,end, from[2]..=191)?;
        }

        // First eq from[0]
        // Second greater then from[1] (if possible)
        // Last can be any valid value.
        {
            if from[1] != 191 {
                let a = nfa.push_state();
                let b = nfa.push_state();

                nfa.push_connection(start,a, from[0])?;
                nfa.push_connections(a,b, from[1]+1..=191)?;
                nfa.push_connections(b,end, 128..=191)?;
            }
        }

        // First eq to[0]
        // Second eq to[1]
        // Last is less then or eq to[2]
        {
            let a = nfa.push_state();
            let b = nfa.push_state();

            nfa.push_connection(start,a, to[0])?;
            nfa.push_connection(a,b, to[1])?;
            nfa.push_connections(b,end, 128..=to[2])?;
        }

        // First eq to[0]
        // Second is less then to[1] (if possible)
        // Last is anything valid.
        {
            if to[1] != 128 {
                let a = nfa.push_state();
                let b = nfa.push_state();

                nfa.push_connection(start,a, to[0])?;
                nfa.push_connections(a,b, 128..to[1])?;
                nfa.push_connections(b,end, 128..=191)?;
            }
        }


        // First is between from[0] and to[0], if that is possible.
        // The last two bytes are anything valid.
        {
            if to[0] - from[0] > 1 { 
                let a = nfa.push_state();
                let b = nfa.push_state();

                nfa.push_connections(start, a, from[0]+1..to[0])?;
                nfa.push_connections(a,b, 128..=191)?;
                nfa.push_connections(b,end, 128..=191)?;
            }
        }
    }

    Ok(())
}

fn between4(from: [u8;4], to: [u8; 4], start: StateId, end: StateId, nfa: &mut NFA) -> Result<(), AutomataBuildError> {
    if from[0] == to[0] {
        let a = nfa.push_state();
        nfa.push_connection(start, a, from[0])?;
        between3([from[1],from[2],from[3]],[to[1],to[2],to[3]], a, end, nfa)?;
    } else {

        // First eq to from[0]
        // Second eq to from[1]
        // Third eq to from[2]
        // Last anything greater then or eq from[3]
        {
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();

            nfa.push_connection(start,a, from[0])?;
            nfa.push_connection(a,b, from[1])?;
            nfa.push_connection(b,c, from[2])?;
            nfa.push_connections(c,end, from[3]..=191)?;
        }


        // First eq to from[0]
        // Second eq to from[1]
        // Third eq is greater then from[2], if possible
        // Last is anything valid
        {
            if  from[2] != 191 {
                let a = nfa.push_state();
                let b = nfa.push_state();
                let c = nfa.push_state();

                nfa.push_connection(start,a,from[0] )?;
                nfa.push_connection(a,b,from[1] )?;
                nfa.push_connections(b,c, from[2]+1..=191)?;
                nfa.push_connections(c,end, 128..=191)?;
            }
        }

        // First eq to from[0]
        // Second is greater then from[1], if possible
        // Last two are anything valid
        {
            if from[1] != 191{
                let a = nfa.push_state();
                let b = nfa.push_state();
                let c = nfa.push_state();

                nfa.push_connection(start,a, from[0])?;
                nfa.push_connections(a,b, from[1]+1..=191)?;
                nfa.push_connections(b,c, 128..=191)?;
                nfa.push_connections(c,end, 128..=191)?;
            }
        }

        // First eq to to[0]
        // Second eq to to[1]
        // Third eq to to[2]
        // Last eq anything less then or eq to[3]
        {
            let a = nfa.push_state();
            let b = nfa.push_state();
            let c = nfa.push_state();

            nfa.push_connection(start,a, to[0])?;
            nfa.push_connection(a,b, to[1])?;
            nfa.push_connection(b,c, to[2])?;
            nfa.push_connections(c,end, 128..=to[3])?;
        }


        // First eq to to[0]
        // Second eq to to[1]
        // Third eq is less then to[2], if possible
        // Last is anything valid
        {
            if  to[2] != 128{
                let a = nfa.push_state();
                let b = nfa.push_state();
                let c = nfa.push_state();

                nfa.push_connection(start,a,to[0])?;
                nfa.push_connection(a,b, to[1])?;
                nfa.push_connections(b,c, 128..=to[2])?;
                nfa.push_connections(c,end, 128..=191)?;
            }
        }

        // First eq to to[0]
        // Second is less then to[1], if possible
        // Last two are anything valid
        {
            if to[1] != 128{
                let a = nfa.push_state();
                let b = nfa.push_state();
                let c = nfa.push_state();

                nfa.push_connection(start,a, to[0])?;
                nfa.push_connections(a,b, 128..=to[1])?;
                nfa.push_connections(b,c,128..=191)?;
                nfa.push_connections(c,end,128..=191)?;
            }
        }


        // First between from[0] and to[0] (if possible)
        // Last three are anything valid
        {
            if to[0] - from[0] > 1 {
                let a = nfa.push_state();
                let b = nfa.push_state();
                let c = nfa.push_state();

                nfa.push_connections(start,a, from[0]+1..to[0])?;
                nfa.push_connections(a,b, 128..=191)?;
                nfa.push_connections(b,c,128..=191)?;
                nfa.push_connections(c,end,128..=191)?;
            }
        }
    }

    Ok(())
}

fn between(mut from: char, mut to: char, start: StateId, end: StateId, nfa: &mut NFA) -> Result<(),AutomataBuildError> {

    // Make it less then or eq other
    if from > to {
        std::mem::swap(&mut from,&mut to);
    }

    let mut fromb = [0u8;4];
    let mut tob = [0u8;4];

    from.encode_utf8(&mut fromb);
    to.encode_utf8(&mut tob);

    match (from.len_utf8(), to.len_utf8()) {
        (1,1) => {
            nfa.push_connections(start,end, fromb[0]..=tob[0])?;
        },
        (1,2) => {
            over_or_eq(from, start, end, nfa)?;
            below_or_eq(to, start, end, nfa)?;
        },
        (1,3) => {
            over_or_eq(from, start, end, nfa)?;
            any_char_of_length_n(2, start, end, nfa)?;
            below_or_eq(to, start, end, nfa)?;
        },
        (1,4) => {
            over_or_eq(from, start, end, nfa)?;
            any_char_of_length_n(2, start, end, nfa)?;
            any_char_of_length_n(3, start, end, nfa)?;
            below_or_eq(to, start, end, nfa)?;
        },
        (2,2) => {
            between2([fromb[0],fromb[1]], [tob[0],tob[1]], start, end, nfa)?;
        },
        (2,3) => {
            over_or_eq(from, start, end, nfa)?;
            below_or_eq(to, start, end, nfa)?;
        },
        (2,4) => {
            over_or_eq(from, start, end, nfa)?;
            any_char_of_length_n(3, start, end, nfa)?;
            below_or_eq(to, start, end, nfa)?;
        },
        (3,3) => {
            between3([fromb[0],fromb[1],fromb[2]],[tob[0],tob[1],tob[2]], start, end, nfa)?;
        },
        (3,4) => {
            over_or_eq(from, start, end, nfa)?;
            below_or_eq(to, start, end, nfa)?;
        },
        (4,4) => {
            between4(fromb, tob, start, end, nfa)?;
        }
        _ => {
            panic!("Not a valid input");
        }
    }

    Ok(())
}

impl From<&ClassUnicodeRange> for NFA {
    fn from(range: &ClassUnicodeRange) -> Self {
        let from = range.start();
        let to = range.end();
        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);
        between(from, to, start, end, &mut nfa).unwrap();

        nfa
    }
}


#[cfg(test)]
mod test {

    use crate::automata::NFA;
    use super::*;
    use core::panic;
    
    const TESTCASES: [char;73] = [
        'a', 'b', 'c', 'd', 'z', // ascii
        'a', 'b', 'c', 'd', 'z', // 2byte
        'ϱ', 'ϯ', 'Ϯ', 'ϯ', 'ϰ', 'ΰ', 'ί', 'ή', 'έ', 'Ͱ', 'ͱ', 'Ͳ',       'Ͱ', //'ඌ', //3 byte
        'ଵ', 'ଶ', 'ଷ', 'ସ', '୵', '୶', '୷', 'வ', 'ஶ', 'ஷ', 'ᬵ', '㬵',       '㭵', '㮵', '㯵', '㬶',
        '㬷', '䬷', //4 byte
        '🚈', '🚉', '🙉', '🙇', '😌', '򟘈', '󟘈', '󜘈', '󄖄',
        // edgecase ascii
        '}', '~', // edgecase 2byte
        'À', 'Á', 'Â', 'Ã', 'ß', 'Þ', 'Ý', '߿', '߾', '޿', // edgecas      e 3byte
        'က', 'ခ', 'ဂ', '၀', 'ႀ', // edgecase 4 byte
        '񀀀', '𿿿', '𿿾', '𿾿', '𾿿', '󿿿',
    ];

    fn any_char_test(c: char, other: char) -> bool {
        // Convert c to byte array
        
        let mut nfa = NFA::default();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);
        
        any_char_of_length_n(c.len_utf8(), start, end, &mut nfa).unwrap();
        
        
        // to string is called and not encode_utf8 because it does not include any
        // trailing zeros.
        let a = nfa.find(c.to_string()).is_ok();
        let b = nfa.find(other.to_string()).is_ok();

        if c.len_utf8() == other.len_utf8() {
            a == true && b == true
        } else {
            b == false && a == true
        }
    }

    #[quickcheck]
    fn qc_any_char_of_length_n(c: char, other: char) -> bool {
        any_char_test(c,other)
    }

    #[test]
    fn test_any_char() {
        for c in TESTCASES.iter() {
            for other in TESTCASES.iter() {
                assert!(any_char_test(*c,*other))
            }
        }
    }

    fn below_char_test(c: char, other: char) -> bool {
        if c.len_utf8() != other.len_utf8() {
            // ignore
            return true;
        }

        let mut nfa = NFA::default();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);
        below_or_eq(c,start,end,&mut nfa).unwrap();

        if other <= c {
            if !(nfa.find(other.to_string()).is_ok()) {
                // Then its a issue
                println!("{}",nfa);
                return false;
            }
        }

        (other <= c) == nfa.find(other.to_string()).is_ok()
    }

    #[quickcheck]
    fn qc_below_char(c: char, other: char) -> bool {
        below_char_test(c,other)
    }
    
     #[test]
    fn test_below_char() {
        for c in TESTCASES.iter() {
            for other in TESTCASES.iter() {
                if !below_char_test(*c,*other) {

                    let mut c_bytes = [0u8;4];
                    let mut other_bytes = [0u8;4];
                    
                    c.encode_utf8(&mut c_bytes);
                    other.encode_utf8(&mut other_bytes);

                }
            }
        }
    }
    
    fn above_char_test(c: char, other: char) -> bool {
        if c.len_utf8() != other.len_utf8() {
            // ignore
            return true;
        }

        let mut nfa = NFA::default();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);
        over_or_eq(c,start,end,&mut nfa).unwrap();

       (c <= other) == nfa.find(other.to_string()).is_ok()
    }

    #[test]
    fn test_over_char() {
        for c in TESTCASES.iter() {
            for other in TESTCASES.iter() {
                if !above_char_test(*c,*other) {
                    panic!("Failed on c:{} {}, other:{} {}",c,c.len_utf8(),other,other.len_utf8());
                }
            }
        }
    }

    fn test_between(from: char, to: char, other: char) -> bool {
        if from > to {
            return true;
        }

        let mut nfa = NFA::empty();
        let start = nfa.push_state();
        let end = nfa.push_state();
        nfa.push_end(end);

        between(from, to, start, end, &mut nfa).unwrap();

        ((from <= other) && ( other <= to)) == nfa.find(other.to_string()).is_ok()  
    }
    
    #[test]
    #[ignore = "Slow"]
    fn test_between_testcases() {
        for from in TESTCASES.iter() {
            for to in TESTCASES.iter() {
                for other in TESTCASES.iter() {

                    if from > to {
                        continue;
                    }

                    if !test_between(*from, *to, *other) {
                        let mut fromb = [0u8;4];
                        let mut tob = [0u8;4];
                        let mut otherb = [0u8;4];
                        from.encode_utf8(&mut fromb);
                        to.encode_utf8(&mut tob);
                        other.encode_utf8(&mut otherb);

                
                        panic!("Failed for from:{} {} {:?}, to: {} {} {:?}, other: {} {} {:?}  {}",from,from.len_utf8(),fromb,to,to.len_utf8(),tob, other, other.len_utf8(),otherb,other < from)
                    }
                }
            }
        }
    }
}
