use enum_iterator::Sequence;


#[derive(Debug, Sequence, PartialEq, Eq, Hash)]
pub enum PacketState {

    Received,
    Prepared,
    PostPrepared,

}


