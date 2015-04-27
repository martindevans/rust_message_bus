extern crate message_bus;

use message_bus::{ Bus, Message };

fn main() {
    let mut bus = Bus::<i32>::new();

    //Send a load of message down the bus
    for i in 0 .. 100 {
        bus.sender.send(Message::<i32>::new(i, i as u64)).unwrap();;
    }

    for i in 0 .. 100 {
        //Tick, publishing received messages
        bus.tick(i);

        //Receive messages from the bus
        print_all(&bus, i as u64);
    }
}

fn print_all(bus : &Bus<i32>, time : u64) {
    loop {
        match bus.receiver.try_recv() {
            Ok(v) => { println!("m{0} @ t{1}", v, time); },
            Err(_) => { break; }
        }
    }
}
