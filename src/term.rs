#![allow(dead_code)]

use std::collections::*;
use net::*;
use std;

// Terms of the Abstract Calculus.
#[derive(Clone, Debug)]
pub enum Term {
    // Abstractions (affine functions).
    Lam {nam: Vec<u8>, bod: Box<Term>},                               

    // Applications.
    App {fun: Box<Term>, arg: Box<Term>},

    // Pairs.
    Par {fst: Box<Term>, snd: Box<Term>},

    // Definitions (let).
    Dup {fst: Vec<u8>, snd: Vec<u8>, val: Box<Term>, nxt: Box<Term>},

    // Variable.
    Var {nam: Vec<u8>}, 

    // Set.
    Set
}
use self::Term::{*};

// Source code is Ascii-encoded.
pub type Str = [u8];
pub type Chr = u8;

// Builds a var name from an index (0="a", 1="b", 26="aa"...).
pub fn new_name(idx : u32) -> Vec<Chr> {
    let mut name = Vec::new();
    let mut idx = idx;
    while idx > 0 {
        idx = idx - 1;
        name.push((97 + idx % 26) as u8);
        idx = idx / 26;
    }
    return name;
}

pub fn name_idx(name : &Vec<Chr>) -> u32 {
    let mut idx : u32 = 0;
    for byte in name.iter().rev() {
        idx = (idx * 26) + (*byte as u32 - 97) + 1;
    }
    return idx;
}

// A context is a vector of (name, value) assignments.
type Context<'a> = Vec<(&'a Str, Option<Term>)>;

// Extends a context with a (name, value) assignments.
fn extend<'a,'b>(nam : &'a Str, val : Option<Term>, ctx : &'b mut Context<'a>) -> &'b mut Context<'a> {
    ctx.push((nam,val));
    ctx
}

// Removes an assignment from a context.
fn narrow<'a,'b>(ctx : &'b mut Context<'a>) -> &'b mut Context<'a> {
    ctx.pop();
    ctx
}


// Parses a name, returns the remaining code and the name.
fn parse_name(code : &Str) -> (&Str, &Str) {
    // Ignore whitespace until name is found. 
    let mut j : usize = 0;
    while j < code.len() && (
        code[j] == b' ' || code[j] == b'\n' || code[j] == b'\r'
    ) {
        if code[j] == b'\\' || code[j] == b'/' || code[j] == b'|' || code[j] == b'=' ||
           code[j] == b'#' || code[j] == b'*'
        { panic!("Valid name not found: {}.", std::str::from_utf8(code).unwrap()) };
        
        j += 1;
    }

    let mut i : usize = j;
    while i < code.len() && !(
        code[i] == b' ' || code[i] == b'\n' || code[i] == b'\r' ||
        code[i] == b'\\' || code[i] == b'/' || code[i] == b'|' || code[i] == b'=' ||
        code[i] == b'#' || code[i] == b'*'
    ) {
        i += 1;
    }

    (&code[i..], &code[j..i])
}

pub fn namespace(space : &Vec<u8>, idx : u32, var : &Vec<u8>) -> Vec<u8> {
    if var != b"-" {
        let mut nam = space.clone();
        nam.extend_from_slice(b"#");
        nam.append(&mut idx.to_string().as_bytes().to_vec());
        nam.extend_from_slice(b"#");
        nam.append(&mut var.clone());
        nam
    } else {
        var.clone()
    }
}

// Makes a namespaced copy of a term
pub fn copy(space : &Vec<u8>, idx : u32, term : &Term) -> Term {
    match term {
        Lam{nam, bod} => {
            let nam = namespace(space, idx, nam);
            let bod = Box::new(copy(space, idx, bod));
            Lam{nam, bod}
        },
        App{fun, arg} => {
            let fun = Box::new(copy(space, idx, fun));
            let arg = Box::new(copy(space, idx, arg));
            App{fun, arg}
        },
        Par{fst, snd} => {
            let fst = Box::new(copy(space, idx, fst));
            let snd = Box::new(copy(space, idx, snd));
            Par{fst, snd}
        },
        Dup{fst, snd, val, nxt} => {
            let fst = namespace(space, idx, fst);
            let snd = namespace(space, idx, snd);
            let val = Box::new(copy(space, idx, val));
            let nxt = Box::new(copy(space, idx, nxt));
            Dup{fst, snd, val, nxt}
        },
        Var{nam} => {
            let nam = namespace(space, idx, nam);
            Var{nam}
        },
        Set => Set
    }
}

// Parses a term, returns the remaining code and the term.
pub fn parse_term<'a>(code : &'a Str, ctx : &mut Context<'a>, idx : &mut u32, comment : u32) -> (&'a Str, Term) {
    if comment > 0 {
        match code[0] {
            b'(' => {
                parse_term(&code[1..], ctx, idx, comment + 1)
            },
            b')' => {
                parse_term(&code[1..], ctx, idx, comment - if comment == 0 { 0 } else { 1 })
            },
            _    => {
                parse_term(&code[1..], ctx, idx, comment)
            }
        }
    } else {
        match code[0] {
            // Whitespace
            b' ' => {
                parse_term(&code[1..], ctx, idx, comment)
            },
            // Newline
            b'\n' => {
                parse_term(&code[1..], ctx, idx, comment)
            },
            // Carriage return
            b'\r' => {
                parse_term(&code[1..], ctx, idx, comment)
            },
            // Comment
            b'(' => {
                parse_term(&code[1..], ctx, idx, comment + 1)
            },
            // Abstraction
            b'\\' => {
                let (code, nam) = parse_name(&code[1..]);
                extend(nam, None, ctx);
                let (code, bod) = parse_term(code, ctx, idx, comment);
                narrow(ctx);
                let nam = nam.to_vec();
                let bod = Box::new(bod);
                (code, Lam{nam,bod})
            },
            // Application
            b'/' => {
                let (code, fun) = parse_term(&code[1..], ctx, idx, comment);
                let (code, arg) = parse_term(code, ctx, idx, comment);
                let fun = Box::new(fun);
                let arg = Box::new(arg);
                (code, App{fun,arg})
            },
            // Pair
            b'|' => {
                let (code, fst) = parse_term(code, ctx, idx, comment);
                let (code, snd) = parse_term(code, ctx, idx, comment);
                let fst = Box::new(fst);
                let snd = Box::new(snd);
                (code, Par{fst,snd})
            },
            // Duplication
            b'=' => {
                let (code, fst) = parse_name(&code[1..]);
                let (code, snd) = parse_name(&code[1..]);
                extend(snd, None, ctx);
                extend(fst, None, ctx);
                let (code, val) = parse_term(code, ctx, idx, comment);
                let (code, nxt) = parse_term(code, ctx, idx, comment);
                narrow(ctx);
                narrow(ctx);
                let fst = fst.to_vec();
                let snd = snd.to_vec();
                let val = Box::new(val);
                let nxt = Box::new(nxt);
                (code, Dup{fst, snd, val, nxt})
            },
            // Definition
            b':' => {
                let (code, nam) = parse_name(&code[1..]);
                let (code, val) = parse_term(code, ctx, idx, comment);
                extend(nam, Some(val), ctx);
                let (code, bod) = parse_term(code, ctx, idx, comment);
                narrow(ctx);
                (code, bod)
            },
            // Set
            b'*' => {
                (&code[1..], Set)
            },
            // Variable
            _ => {
                let (code, nam) = parse_name(code);
                let mut val : Option<Term> = None;
                for i in (0..ctx.len()).rev() {
                    if ctx[i].0 == nam {
                        match ctx[i].1 {
                            Some(ref term) => {
                                let name = nam.clone().to_vec();
                                val = Some(copy(&name, *idx, term));
                                *idx += 1;
                                break;
                            },
                            None => {
                                break;
                            }
                        }
                    }
                }
                let nam = nam.to_vec();
                (code, match val { Some(term) => term, None => Var{nam} })
            }
        }
    }
}

// Converts a source-code to a λ-term.
pub fn from_string<'a>(code : &'a Str) -> Term {
    let mut ctx = Vec::new();
    let mut idx = 0;
    parse_term(code, &mut ctx, &mut idx, 0).1
}

// Converts a λ-term back to a source-code.
pub fn to_string(term : &Term) -> Vec<Chr> {
    fn stringify_term(code : &mut Vec<u8>, term : &Term) {
        match term {
            &Lam{ref nam, ref bod} => {
                code.extend_from_slice(b"\\");
                code.append(&mut nam.clone());
                code.extend_from_slice(b" ");
                stringify_term(code, &bod);
            },
            &App{ref fun, ref arg} => {
                code.extend_from_slice(b"/");
                stringify_term(code, &fun);
                code.extend_from_slice(b" ");
                stringify_term(code, &arg);
            },
            &Par{ref fst, ref snd} => {
                code.extend_from_slice(b"|");
                code.extend_from_slice(b" ");
                stringify_term(code, &fst);
                code.extend_from_slice(b" ");
                stringify_term(code, &snd);
            },
            &Dup{ref fst, ref snd, ref val, ref nxt} => {
                code.extend_from_slice(b"=");
                code.extend_from_slice(b" ");
                code.append(&mut fst.clone());
                code.extend_from_slice(b" ");
                code.append(&mut snd.clone());
                code.extend_from_slice(b" ");
                stringify_term(code, &val);
                code.extend_from_slice(b"\n");
                stringify_term(code, &nxt);
            },
            &Set => {
                code.extend_from_slice(b"*");
            },
            &Var{ref nam} => {
                code.append(&mut nam.clone());
            },
        }
    }
    let mut code = Vec::new();
    stringify_term(&mut code, &term);
    return code;
}

// Display macro.
impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&to_string(&self)))
    }
}

// Converts a term to an Interaction Combinator net. Both systems are directly isomorphic, so,
// each node of the Abstract Calculus correspond to a single Interaction Combinator node.
pub fn to_net(term : &Term) -> Net {
    fn encode_term
        ( net   : &mut Net
        , term  : &Term
        , up    : Link
        , scope : &mut HashMap<Vec<u8>,u32>
        , vars  : &mut Vec<(Vec<u8>,u32)>
        ) -> Link {
        match term {
            // A lambda becomes to a con node. Ports:
            // - 0: points to where the lambda occurs.
            // - 1: points to the lambda variable.
            // - 2: points to the lambda body.
            &Lam{ref nam, ref bod} => {
                let fun = new_node(net, CON);
                scope.insert(nam.to_vec(), link(fun, 1));
                // Also, if the variable is unused, crease an erase node.
                if nam == b"_" {
                    let era = new_node(net, ERA);
                    connect(net, link(era, 1), link(era, 2));
                    connect(net, link(fun, 1), link(era, 0));
                }
                let bod = encode_term(net, bod, link(fun, 2), scope, vars);
                connect(net, link(fun, 2), bod);
                link(fun, 0)
            },
            // An application becomes to a con node too. Ports:
            // - 0: points to the function being applied.
            // - 1: points to the function's argument.
            // - 2: points to where the application occurs.
            &App{ref fun, ref arg} => {
                let app = new_node(net, CON);
                let fun = encode_term(net, fun, link(app, 0), scope, vars);
                connect(net, link(app, 0), fun);
                let arg = encode_term(net, arg, link(app, 1), scope, vars);
                connect(net, link(app, 1), arg);
                link(app, 2)
            },
            // A pair becomes a dup node. Ports:
            // - 0: points to where the pair occurs.
            // - 1: points to the first value.
            // - 2: points to the second value.
            &Par{ref fst, ref snd} => {
                let dup = new_node(net, FAN);
                let fst = encode_term(net, fst, link(dup, 1), scope, vars);
                connect(net, link(dup, 1), fst);
                let snd = encode_term(net, snd, link(dup, 2), scope, vars);
                connect(net, link(dup, 2), snd);
                link(dup, 0)
            },
            // A duplication becomes a dup node too. Ports:
            // - 0: points to the value projected.
            // - 1: points to the occurrence of the first variable.
            // - 2: points to the occurrence of the second variable.
            &Dup{ref fst, ref snd, ref val, ref nxt} => {
                let dup = new_node(net, FAN);
                scope.insert(fst.to_vec(), link(dup, 1));
                scope.insert(snd.to_vec(), link(dup, 2));
                // If the first variable is unused, create an erase node.
                if fst == b"-" {
                    let era = new_node(net, ERA);
                    connect(net, link(era, 1), link(era, 2));
                    connect(net, link(dup, 1), link(era, 0));
                }
                // If the second variable is unused, create an erase node.
                if snd == b"-" {
                    let era = new_node(net, ERA);
                    connect(net, link(era, 1), link(era, 2));
                    connect(net, link(dup, 2), link(era, 0));
                }
                let val = encode_term(net, &val, link(dup, 0), scope, vars);
                connect(net, val, link(dup, 0));
                encode_term(net, &nxt, up, scope, vars)
            },
            // A set is just an erase node stored in a place.
            &Set => {
                let set = new_node(net, ERA);
                connect(net, link(set, 1), link(set, 2));
                link(set, 0)
            },
            Var{ref nam} => {
                vars.push((nam.to_vec(), up));
                up
            }
        }
    }

    // Initializes net with a root node.
    let mut net = Net { nodes: vec![0,2,1,4], reuse: vec![] };
    let mut vars = Vec::new();
    let mut scope = HashMap::new();

    // Encodes the main term.
    let main = encode_term(&mut net, &term, 0, &mut scope, &mut vars);

    // Links bound variables.
    for i in 0..vars.len() {
        let (ref nam, var) = vars[i];
        match scope.get(nam) {
            Some(next) => {
                let next = *next;
                if enter(&net, next) == next {
                    connect(&mut net, var, next);
                } else {
                    panic!("Variable used more than once: {}.", std::str::from_utf8(nam).unwrap());
                }
            },
            None => panic!("Unbound variable: {}.", std::str::from_utf8(nam).unwrap())
        }
    }

    // Connects unbound variables to erase nodes
    for (_, addr) in scope {
        if enter(&net, addr) == addr {
            let era = new_node(&mut net, ERA);
            connect(&mut net, link(era, 1), link(era, 2));
            connect(&mut net, addr, link(era, 0));
        }
    }

    // Links the term to the net's root.
    connect(&mut net, 0, main);

    net
}

// Converts an Interaction-Net node to an Abstract Calculus term.
pub fn from_net(net : &Net) -> Term {
    // Given a link, returns its name, or assigns one if it wasn't named yet.
    fn name_of(net : &Net, var_port : Link, var_name : &mut HashMap<u32, Vec<u8>>) -> Vec<u8> {
        // If link is linked to an erase node, return an unused variable
        if kind(net, addr(enter(net, var_port))) == ERA {
            return b"-".to_vec();
        }
        if !var_name.contains_key(&var_port) {
            let nam = new_name(var_name.len() as u32 + 1);
            var_name.insert(var_port, nam.clone());
        }
        var_name.get(&var_port).unwrap().to_vec()
    }

    // Reads a term recursively by starting at root node.
    fn read_term
        ( net      : &Net
        , next     : Link
        , var_name : &mut HashMap<u32, Vec<u8>>
        , lets_vec : &mut Vec<u32>
        , lets_set : &mut HashSet<u32>
        ) -> Term {
        match kind(net, addr(next)) {
            // If we're visiting a set...
            ERA => Set,
            // If we're visiting a con node...
            CON => match port(next) {
                // If we're visiting a link 0, then it is a lambda.
                0 => {
                    let nam = name_of(net, link(addr(next),1), var_name);
                    let prt = enter(net, link(addr(next), 2));
                    let bod = read_term(net, prt, var_name, lets_vec, lets_set);
                    let lam = Lam{nam: nam, bod: Box::new(bod)};
                    lam
                },
                // If we're visiting a link 1, then it is a variable.
                1 => {
                    Var{nam: name_of(net, next, var_name)}
                },
                // If we're visiting a link 2, then it is an application.
                _ => {
                    let prt = enter(net, link(addr(next), 0));
                    let fun = read_term(net, prt, var_name, lets_vec, lets_set);
                    let prt = enter(net, link(addr(next), 1));
                    let arg = read_term(net, prt, var_name, lets_vec, lets_set);
                    App{fun: Box::new(fun), arg: Box::new(arg)}
                }
            },
            // If we're visiting a fan node...
            FAN => match port(next) {
                // If we're visiting a link 0, then it is a pair.
                0 => {
                    let prt = enter(net, link(addr(next), 1));
                    let fst = read_term(net, prt, var_name, lets_vec, lets_set);
                    let prt = enter(net, link(addr(next), 2));
                    let snd = read_term(net, prt, var_name, lets_vec, lets_set);
                    Par{fst: Box::new(fst), snd: Box::new(snd)}
                },
                // If we're visiting a link 1 or 2, then it is a variable.
                // Also, that means we found a let, so we store it to read later.
                _ => {
                    if !lets_set.contains(&addr(next)) {
                        lets_set.insert(addr(next));
                        lets_vec.push(addr(next));
                    }
                    let nam = name_of(net, next, var_name);
                    Var{nam}
                }
            },
            _ => panic!("Unknown kind of node"),
        }
    }

    // A hashmap linking ports to binder names. Those ports have names:
    // Link 1 of a con node (λ), ports 1 and 2 of a fan node (let).
    let mut binder_name = HashMap::new();

    // Lets aren't scoped. We find them when we read one of the variables
    // introduced by them. Thus, we must store the lets we find to read later.
    // We have a vec for .pop(). and a set to avoid storing duplicates.
    let mut lets_vec = Vec::new();
    let mut lets_set = HashSet::new();

    // Reads the main term from the net
    let mut main = read_term(net, enter(net, 0), &mut binder_name, &mut lets_vec, &mut lets_set);

    // Reads let founds by starting the read_term function from their 0 ports.
    while lets_vec.len() > 0 {
        let dup = lets_vec.pop().unwrap();
        let val = read_term(net, enter(net,link(dup,0)), &mut binder_name, &mut lets_vec, &mut lets_set);
        let fst = name_of(net, link(dup,1), &mut binder_name);
        let snd = name_of(net, link(dup,2), &mut binder_name);
        let val = Box::new(val);
        let nxt = Box::new(main);
        main = Dup{fst, snd, val, nxt};
    }
    main
}

// Reduces an Abstract Calculus term through Interaction Combinators.
pub fn reduce(term : &Term) -> Term {
    let mut net : Net = to_net(&term);
    ::net::reduce(&mut net);
    from_net(&net)
}
