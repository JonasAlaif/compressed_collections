use compressed_collections::ChunkSize;
use compressed_collections::Deque;
use compressed_collections::Stack;

#[test]
fn stack_test() {
    let mut big_vec = Vec::new();
    let mut compressed_stack = Stack::new_with_options(ChunkSize::SizeElements(1024 * 9));
    for x in 0..(1024 * 10) {
        let y = (x % 5) as f64;
        big_vec.push(y);
        compressed_stack.push(y);
    }
    loop {
        let a = big_vec.pop();
        let b = compressed_stack.pop();
        assert!(a == b);
        if a.is_none() | b.is_none() {
            break;
        }
    }
}

#[test]
fn deque_test() {
    let mut big_vecdeque = std::collections::VecDeque::new();
    let mut compressed_deque = Deque::new_with_options(ChunkSize::SizeElements(1024 * 9));
    for _ in 0..(1024 * 10) {
        big_vecdeque.push_back(1);
        compressed_deque.push_back(1);
    }
    loop {
        let a = big_vecdeque.pop_front();
        let b = compressed_deque.pop_front();
        assert!(a == b);
        if a.is_none() | b.is_none() {
            break;
        }
    }
}

#[test]
fn deque_test_2() {
    let mut big_vecdeque = std::collections::VecDeque::new();
    let mut compressed_deque = Deque::new_with_options(ChunkSize::SizeElements(1024 * 1));
    for _ in 0..(1024 * 10) {
        big_vecdeque.push_front(1);
        compressed_deque.push_front(1);
    }
    loop {
        let a = big_vecdeque.pop_back();
        let b = compressed_deque.pop_back();
        assert!(a == b);
        if a.is_none() | b.is_none() {
            break;
        }
    }
}

#[test]
fn deque_test_3() {
    let mut big_vecdeque = std::collections::VecDeque::new();
    let mut compressed_deque = Deque::new_with_options(ChunkSize::SizeElements(1024 * 1));
    for _ in 0..(1024 * 10) {
        big_vecdeque.push_front(1);
        compressed_deque.push_front(1);
    }
    loop {
        let a = big_vecdeque.pop_front();
        let b = compressed_deque.pop_front();
        assert!(a == b);
        if a.is_none() | b.is_none() {
            break;
        }
    }
}

#[test]
fn deque_test_4() {
    let mut big_vecdeque = std::collections::VecDeque::new();
    let mut compressed_deque = Deque::new_with_options(ChunkSize::SizeElements(1024 * 1));
    for _ in 0..(1024 * 10) {
        big_vecdeque.push_back(1);
        compressed_deque.push_back(1);
    }
    loop {
        let a = big_vecdeque.pop_back();
        let b = compressed_deque.pop_back();
        assert!(a == b);
        if a.is_none() | b.is_none() {
            break;
        }
    }
}

#[test]
fn deque_test_5() {
    let mut big_vecdeque = std::collections::VecDeque::new();
    let mut compressed_deque = Deque::new_with_options(ChunkSize::SizeElements(1024 * 1));
    for x in 0..(1024 * 10) {
        let y = x % 7;
        big_vecdeque.push_back(y);
        compressed_deque.push_back(y);
    }
    for _ in 0..(1024 * 4) {
        let a = big_vecdeque.pop_back();
        let b = compressed_deque.pop_back();
        assert!(a == b);
        if a.is_none() | b.is_none() {
            break;
        }
    }
    for x in 0..(1024 * 10) {
        let y = x % 11;
        big_vecdeque.push_front(y);
        compressed_deque.push_front(y);
    }
    for _ in 0..(1024 * 4) {
        let a = big_vecdeque.pop_front();
        let b = compressed_deque.pop_front();
        assert!(a == b);
        if a.is_none() | b.is_none() {
            break;
        }
    }
    for x in 0..(1024 * 10) {
        let y = x % 13;
        big_vecdeque.push_back(y);
        compressed_deque.push_back(y);
    }
    loop {
        let a = big_vecdeque.pop_front();
        let b = compressed_deque.pop_front();
        assert!(a == b);
        if a.is_none() | b.is_none() {
            break;
        }
    }
}
