use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

// A message bus which buffers messages and sends them at the correct timestamp
// When a message is sent it is sent with a timestamp Message::<valuetype>::new(value, receive_time)
// The message will not be received by anything until bus.tick(time) where time >= receive_time given above
// When receive_time has passed the message will be sent to bus.output_receiver
pub struct Bus<T> {
    pub sender: Sender<Message<T>>,
    input_receiver: Receiver<Message<T>>,

    output_sender: Sender<T>,
    pub receiver: Receiver<T>,

    buffers: [Vec<Message<T>>; 32]
}

impl<T> Bus<T> {
    pub fn new() -> Bus<T> {
        //Input channel
        let (itx, irx) = mpsc::channel();

        //output channel
        let (otx, orx) = mpsc::channel();

        return Bus {
            sender: itx,
            input_receiver: irx,

            output_sender: otx,
            receiver: orx,

            //A set of buffers for messages in the future
            //Each buffer will be checked every 2^index ticks, messages are put into the farthest buffer possible
            //This messages messages will be shuffled log2(number of ticks into the future) times
            buffers: [
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
                Vec::<Message<T>>::new(),
            ]
        };
    }

    pub fn tick(&mut self, time : u64) {
        // Pump input messages, either into buffers or straight into the output
        loop {
            match self.input_receiver.try_recv() {
                Ok(v) => {
                    if v.time <= time {
                        self.output_sender.send(v.message).unwrap();;
                    } else {
                        self.buffers[buffer_index_for_time_offset(v.time - time)].push(v);
                    }
                },
                Err(_) => { break; }
            }
        }

        // Check buffers
        for buffer_index in 0 .. self.buffers.len() {
            if time % ((buffer_index + 1) as u64) == 0 {

                //Iterate through buffer, pumping appropriate messages
                let size = self.buffers[buffer_index].len();
                for j in 0 .. size {

                    //Index of the message in the buffer (we need to count backwards, since we're modifying the buffer as we step through it)
                    let msg_index = size - j - 1;

                    //timestamp of the message
                    let msg_time = self.buffers[buffer_index][msg_index].time;

                    //Either send this message, or move it to a lower buffer
                    if msg_time <= time {
                        self.output_sender.send(self.buffers[buffer_index].swap_remove(msg_index).message).unwrap();
                    } else {
                        //Calculate which buffer the message should be in, given the time offset
                        let move_to_buffer_index = buffer_index_for_time_offset(msg_time - time);
                        if move_to_buffer_index != buffer_index {
                            let msg = self.buffers[buffer_index].swap_remove(msg_index);
                            self.buffers[move_to_buffer_index].push(msg);
                        }
                    }
                }
            }
        }
    }
}

fn buffer_index_for_time_offset(offset : u64) -> usize {
    return (offset.next_power_of_two().trailing_zeros()) as usize;
}

pub struct Message<T> {
    message: T,
    time: u64
}

impl<T> Message<T> {
    pub fn new(message: T, time: u64) -> Message<T> {
        return Message {
            message: message,
            time: time
        }
    }
}

#[test]
fn check_buffer_index_for_times_is_calculated_correctly() {
    assert_eq!(buffer_index_for_time_offset(0) , 0);
    assert_eq!(buffer_index_for_time_offset(1) , 0);
    assert_eq!(buffer_index_for_time_offset(2) , 1);
    assert_eq!(buffer_index_for_time_offset(3) , 2);
    assert_eq!(buffer_index_for_time_offset(4) , 2);
    assert_eq!(buffer_index_for_time_offset(5) , 3);
    assert_eq!(buffer_index_for_time_offset(100) , 7);
    assert_eq!(buffer_index_for_time_offset(500) , 9);
    assert_eq!(buffer_index_for_time_offset(750) , 10);
    assert_eq!(buffer_index_for_time_offset(1000) , 10);
    assert_eq!(buffer_index_for_time_offset(1025) , 11);
}

#[test]
fn check_messages_are_returned_at_correct_time() {
    //Create a bus
    let mut bus = Bus::<u64>::new();

    //Send messages down bus, each message contains the time it should be received
    for i in 0 .. 1000 {
        bus.sender.send(Message::<u64>::new(i as u64, i as u64));
        bus.sender.send(Message::<u64>::new(i as u64, i as u64));
        bus.sender.send(Message::<u64>::new(i as u64, i as u64));
    }

    for i in 0 .. 1000 {
        //Tick, publishing received messages
        bus.tick(i);

        //Receive messages from the bus
        loop {
            match bus.receiver.try_recv() {
                Ok(v) => { assert_eq!(v, i); },
                Err(_) => { break; }
            }
        }
    }
}
