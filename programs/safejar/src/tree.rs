use std::cell::RefCell;
use std::io::{Cursor, Read};
use std::rc::Rc;

use anchor_lang::error_code;
use anchor_lang::solana_program::msg;

use crate::nplog;

#[error_code]
pub enum BooleanTreeError {
    #[msg("unknown error")]
    Unknown,
    #[msg("failed to read")]
    FailedToRead,
    #[msg("Bad Boolean")]
    BadBoolean,
    #[msg("Index out of range")]
    IndexOutOfRange,
}

#[derive(Clone)]
pub struct Node {
    i: u8,
    is_and: bool,
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new_with_children(
        is_and: bool,
        left: &Rc<RefCell<Node>>,
        right: &Rc<RefCell<Node>>,
    ) -> Self {
        Self {
            i: NULL,
            is_and,
            left: Some(left.clone()),
            right: Some(right.clone()),
        }
    }
    pub fn new() -> Self {
        Node {
            i: NULL,
            is_and: true,
            left: None,
            right: None,
        }
    }

    pub fn set_i(&mut self, i: u8) {
        self.i = i;
        self.is_and = false; // this value does not matter
        self.left = None;
        self.right = None;
    }
    pub fn set_and(&mut self) {
        self.is_and = true;
    }
    pub fn set_or(&mut self) {
        self.is_and = false;
    }
    pub fn set_left(&mut self, node: &Option<Rc<RefCell<Node>>>) {
        self.i = NULL;
        self.left = node.clone();
    }
    pub fn set_right(&mut self, node: &Option<Rc<RefCell<Node>>>) {
        self.i = NULL;
        self.right = node.clone();
    }

    pub fn serialize(&self) -> Vec<u8> {
        let n = Rc::new(RefCell::new(self.clone()));
        return serialize(Some(n));
    }
}

pub const NULL: u8 = u8::MAX;

pub fn serialize(root: Option<Rc<RefCell<Node>>>) -> Vec<u8> {
    let mut a = Vec::new();
    serialize_helper(root, &mut a);
    return a;
}

fn serialize_helper(root: Option<Rc<RefCell<Node>>>, wtr: &mut Vec<u8>) {
    match root {
        Some(node) => {
            let val = node.borrow().i;
            wtr.push(val);
            if val == NULL {
                if node.borrow().is_and {
                    wtr.push(1)
                } else {
                    wtr.push(0)
                }
                serialize_helper(node.borrow().left.clone(), wtr);
                serialize_helper(node.borrow().right.clone(), wtr);
            }
        }
        None => {
            //        wtr.push(NULL);
        }
    }
}

fn get_result(result: &u64, index: &u8) -> bool {
    nplog!("get_result - index {} result {:064b} shift", index, result,);
    return 0 < *result & (1 << *index);
}

pub(crate) fn evaluate(count: &u8, root: Option<Rc<RefCell<Node>>>, result: &u64) -> bool {
    match root {
        Some(x) => {
            return evaluate_helper(count, x, result);
        }
        None => {
            let i = 0;
            return get_result(result, &i);
        }
    }
}

fn evaluate_helper(count: &u8, root: Rc<RefCell<Node>>, result: &u64) -> bool {
    let node = root.borrow();
    let i = node.i;
    if i < NULL {
        nplog!("tree eval - 1 - i {} result {:064b} ", i, result);
        return get_result(result, &i);
    } else {
        let left = evaluate_helper(count, node.left.clone().unwrap(), result);
        let right = evaluate_helper(count, node.right.clone().unwrap(), result);
        nplog!(
            "tree eval - 2 - i={} is_and {} left {} right {}",
            i,
            node.is_and,
            left,
            right
        );
        if node.is_and {
            return left && right;
        } else {
            return left || right;
        }
    }
}

pub fn deserialize(data: &Vec<u8>) -> Result<(Option<Rc<RefCell<Node>>>, u8), BooleanTreeError> {
    let mut cursor = Cursor::new(data.clone());
    let mut max = 0;
    let mut count = 0;
    let ans = deserialize_helper(None, &mut cursor, &mut max, &mut count)?;

    nplog!("data len={} vs cursor={}", data.len(), cursor.position());
    if (cursor.position() as usize) < data.len() {
        nplog!("index out of range");
        return Err(BooleanTreeError::IndexOutOfRange.into());
    }
    if count < max {
        nplog!("count={} vs max={}", count, max);
        return Err(BooleanTreeError::Unknown.into());
    }
    nplog!("finished deserialize");

    return Ok((ans, count));
}

fn deserialize_helper(
    parent: Option<Rc<RefCell<Node>>>,
    rdr: &mut Cursor<Vec<u8>>,
    max: &mut u8,
    count: &mut u8,
) -> Result<Option<Rc<RefCell<Node>>>, BooleanTreeError> {
    let n;
    match parent {
        Some(x) => {
            n = x;
        }
        None => {
            //nplog!("dh - new n");
            n = Rc::new(RefCell::new(Node::new()));
        }
    }

    let mut buf: [u8; 1] = [0; 1];
    match rdr.read_exact(&mut buf) {
        Ok(_) => {
            //nplog!("dh - 1 - n with i={}", n.borrow().i);
            n.borrow_mut().i = buf[0];
        }
        Err(_e) => {
            //nplog!("dh - 1 - failed to read");
            return Err(BooleanTreeError::FailedToRead.into());
        }
    }
    let current_value = n.borrow().i;
    if current_value == NULL {
        // nplog!("dh - 2 - i=null");
        match rdr.read_exact(&mut buf) {
            Ok(_) => {
                nplog!("dh - 6");
                if buf[0] == 0 {
                    n.borrow_mut().is_and = false;
                } else if buf[0] == 1 {
                    n.borrow_mut().is_and = true;
                } else {
                    nplog!("dh - 7");
                    return Err(BooleanTreeError::BadBoolean.into());
                }
            }
            Err(_e) => {
                nplog!("dh - 8");
                return Err(BooleanTreeError::Unknown.into());
            }
        }

        let left = Rc::new(RefCell::new(Node::new()));
        n.borrow_mut().left = Some(left.clone());
        //nplog!("dh - 9 - left - parent={}", current_value);
        deserialize_helper(Some(left), rdr, max, count)?;
        let right = Rc::new(RefCell::new(Node::new()));
        n.borrow_mut().right = Some(right.clone());
        //nplog!("dh - 9 - right - parent={}", current_value);
        deserialize_helper(Some(right), rdr, max, count)?;
        // nplog!("dh - 10 - finished n={}", current_value);
    } else {
        if *max < current_value {
            *max = current_value;
        }
        *count += 1;
    }
    Ok(Some(n))
}
