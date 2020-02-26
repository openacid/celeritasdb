macro_rules! test_enc_dec {
    // $msg is a prost Message.
    // $typ is one of message.Request or message.Reply
    ($msg:ident, $typ:ty) => {
        let size = $msg.encoded_len();
        assert!(size > 0);

        let mut byts = vec![];
        $msg.encode(&mut byts).unwrap();
        let msg2 = <$typ>::decode(byts.as_slice()).unwrap();
        assert_eq!($msg, msg2);
    };
}
