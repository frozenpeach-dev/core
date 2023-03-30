use enum_iterator::Sequence;


#[derive(Copy, Debug, Sequence, PartialEq, Eq, Hash, Clone)]
pub enum PacketState {

    Received,
    Prepared,
    PostPrepared,

}


