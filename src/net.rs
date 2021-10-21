// Implements Interaction Combinators. The Abstract Calculus is directly isomorphic to them, so, to
// reduce a term, we simply translate to interaction combinators, reduce, then translate back.

#![allow(dead_code)]

#[derive(Clone, Debug)]
pub struct Stats {
    pub loops: u32,
    pub rules: u32,
    pub betas: u32,
    pub dupls: u32,
    pub annis: u32
}

#[derive(Clone, Debug)]
pub struct Net {
    pub nodes: Vec<u32>,
    pub reuse: Vec<u32>
}

// Node types are consts because those are used in a Vec<u32>.
pub const ERA : u32 = 0;
pub const CON : u32 = 1;
pub const FAN : u32 = 2;

pub type Link = u32;

// Allocates a new node, reclaiming a freed space if possible.
pub fn new_node(net : &mut Net, kind : u32) -> u32 {
    let node : u32 = match net.reuse.pop() {
        Some(index) => index,
        None => {
            let len = net.nodes.len();
            net.nodes.resize(len + 4, 0);
            (len as u32) / 4
        }
    };
    net.nodes[link(node, 0) as usize] = link(node, 0);
    net.nodes[link(node, 1) as usize] = link(node, 1);
    net.nodes[link(node, 2) as usize] = link(node, 2);
    net.nodes[link(node, 3) as usize] = kind;
    return node;
}

// Builds a link (an address / port pair).
pub fn link(node : u32, port : u32) -> Link {
    (node << 2) | port
}

// Returns the address of a link.
pub fn addr(link : Link) -> u32 {
    link >> 2
}

// Returns the port of a link.
pub fn port(link : Link) -> u32 {
    link & 3
}

// Enters a link, returning the link on the other side.
pub fn enter(net : &Net, link : Link) -> Link {
    net.nodes[link as usize]
}

// Type of the node.
// 0 = era (i.e., a set or a garbage collector)
// 1 = con (i.e., a lambda or an application)
// 2 = fan (i.e., a pair or a let)
pub fn kind(net : &Net, node : u32) -> u32 {
    net.nodes[link(node, 3) as usize]
}

// Connect two ports.
pub fn connect(net : &mut Net, ptr_a : u32, ptr_b : u32) {
    net.nodes[ptr_a as usize] = ptr_b;
    net.nodes[ptr_b as usize] = ptr_a;
}

// Reduces a net to normal form lazily and sequentially.
pub fn reduce(net : &mut Net) -> Stats {
    let mut stats = Stats { loops: 0, rules: 0, betas: 0, dupls: 0, annis: 0 };
    let mut schedule : Vec<u32> = Vec::new();
    let mut exit : Vec<u32> = Vec::new();
    let mut next : Link = net.nodes[0];
    let mut prev : Link;
    let mut back : Link;
    while next > 0 || schedule.len() > 0 {
        next = if next == 0 { enter(net, schedule.pop().unwrap()) } else { next };
        prev = enter(net, next);
        if port(next) == 0 && port(prev) == 0 && addr(prev) != 0 {
            stats.rules += 1;
            back = enter(net, link(addr(prev), exit.pop().unwrap()));
            rewrite(net, addr(prev), addr(next));
            next = enter(net, back);
        } else if port(next) == 0 {
            schedule.push(link(addr(next), 2));
            next = enter(net, link(addr(next), 1));
        } else {
            exit.push(port(next));
            next = enter(net, link(addr(next), 0));
        }
        stats.loops += 1;
    }
    stats
}

// Rewrites an active pair.
pub fn rewrite(net : &mut Net, x : Link, y : Link) {
    if kind(net, x) == kind(net, y) {
        let p0 = enter(net, link(x, 1));
        let p1 = enter(net, link(y, 1));
        connect(net, p0, p1);
        let p0 = enter(net, link(x, 2));
        let p1 = enter(net, link(y, 2));
        connect(net, p0, p1);
        net.reuse.push(x);
        net.reuse.push(y);
    } else {
        let t = kind(net, x);
        let a = new_node(net, t);
        let t = kind(net, y);
        let b = new_node(net, t);
        let t = enter(net, link(x, 1));
        connect(net, link(b, 0), t);
        let t = enter(net, link(x, 2));
        connect(net, link(y, 0), t);
        let t = enter(net, link(y, 1));
        connect(net, link(a, 0), t);
        let t = enter(net, link(y, 2));
        connect(net, link(x, 0), t);
        connect(net, link(a, 1), link(b, 1));
        connect(net, link(a, 2), link(y, 1));
        connect(net, link(x, 1), link(b, 2));
        connect(net, link(x, 2), link(y, 2));
    }
}

pub fn print_net(net : &mut Net) {
    let mut i = 0;

    while i < net.nodes.len() {
        println!("{}: {}.{} | {}.{} | {}.{} | K:{}", i >> 2, 
            port(net.nodes[i]), addr(net.nodes[i]), 
            port(net.nodes[i+1]), addr(net.nodes[i+1]), 
            port(net.nodes[i+2]), addr(net.nodes[i+2]), 
            net.nodes[i+3]
        );
        i+=4;
    }

    println!("Empty addresses:");

    i=0;
    while i < net.reuse.len() {
        println!("{}", net.reuse[i] );
        i+=1;
    }
}
