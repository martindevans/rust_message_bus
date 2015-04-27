This is an implementation of a "message bus" which buffers messages and sends them at the correct timestamp. When a message is sent it is sent with a timestamp, e.g.

    bus.sender.send(Message::<valuetype>::new(value, receive_time));

The message will not be received by anything until that time arrives. i.e.

    bus.tick(time);

Where time >= receive_time given when sending. When receive_time has arrived the message will be sent to bus.output_receiver
