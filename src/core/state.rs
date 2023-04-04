use enum_iterator::Sequence;


#[derive(Debug, Sequence, Clone, Copy)]
pub enum PacketState {

    Received,
    Prepared,
    PostPrepared,

}


