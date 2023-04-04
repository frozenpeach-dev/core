//! Defines an abstract representation of the data
//! that can be processed, through a [`PacketType`]
//!
//! These types are then encapsulated in a 
//! [`PacketContext`], which will be enriched by the
//! [`Hook`] to create a valid output packet.

use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use super::state::PacketState;

pub trait PacketType {
    fn to_raw_bytes(&self) -> &[u8];
    fn empty() -> Self;
    fn from_raw_bytes() -> Self;
}

/// A `PacketContext` encapsulates two things:
/// - An input packet, used to derive the [`PacketContext`]
/// - An output packet, which is initially empty and is
/// enriched with data through execution of [`Hook`]
///
/// It is identified uniquely across the program using its [`Uuid`],
/// and it holds a [`PacketState`]. Through [`Hook`] executions, it 
/// will undergo several successive state transitions.

pub struct PacketContext<T : PacketType, U: PacketType> {

    time: DateTime<Utc>,
    id: Uuid,
    state: PacketState,
    input_packet : T,
    output_packet : U

}

impl<T: PacketType, U: PacketType> PacketContext<T, U> {

    /// Returns the [`Uuid`] of the PacketContext
    ///
    /// # Examples:
    ///
    /// ```
    /// let a = PacketContext::from(packet);
    /// println!(a.id());
    /// ```
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the current [`PacketState`] associated
    /// to the packet.
    ///
    /// # Examples:
    ///
    /// ```
    /// let a = PacketContext::from(packet);
    /// assert!(a.state() == PacketState::Received);
    /// ```
    pub fn state(&self) -> PacketState {
        self.state
    }

    /// Set the current [`PacketState`] associated
    /// to the packet
    ///
    /// # Examples:
    ///
    /// ```
    /// let a = PacketContext::from(packet);
    /// a.set_state(PacketState::Prepared);
    /// assert!(a.state() == PacketState::Prepared);
    /// ```
    pub fn set_state(&mut self, new_state: PacketState) {
        self.state = new_state;
    }

    /// Returns the current output packet contained
    /// in the context
    ///
    /// #Examples:
    ///
    /// ```
    /// let a = PacketContext<A, A>::from(packet);
    /// assert!(a.get_output() == A::empty());
    /// ```
    pub fn get_output(&self) -> &U {
        &self.output_packet
    }
    /// Returns the current input packet contained
    /// in the context
    ///
    /// # Examples:
    ///
    /// ```
    /// let a = PacketContext<A, A>::from(packet);
    /// assert!(a.get_input() == packet);
    /// ```
    pub fn get_input(&self) -> &T {
        &self.input_packet
    }
    /// Returns a mutable reference to the 
    /// current output packet contained in the context
    ///
    /// # Examples:
    ///
    /// ```
    /// let a = PacketContext<A, A>::from(packet);
    /// assert!(a.get_mut_output() == A::empty());
    /// ```
    pub fn get_mut_output(&mut self) -> &mut U {
        &mut self.output_packet
    }
    /// Returns a mutable reference to the 
    /// current input packet contained in the context
    ///
    /// # Examples:
    ///
    /// ```
    /// let a = PacketContext<A, A>::from(packet);
    /// assert!(a.get_mut_input() == packet);
    /// ```
    pub fn get_mut_input(&mut self) -> &mut T {
        &mut self.input_packet
    }

    /// Converts the contained input packet 
    /// to its raw bytes representation
    pub fn input_to_raw(&self) -> &[u8] {
        self.input_packet.to_raw_bytes()
    }

    /// Converts the contained output packet 
    /// to its raw bytes representation
    pub fn output_to_raw(&self) -> &[u8]{
        self.output_packet.to_raw_bytes()
    }

    /// Returns the current lifetime of
    /// the [`PacketContext`], as a [`Duration`]
    pub fn lifetime(&self) -> Duration{
        Utc::now() - self.time
    }

}

impl<T: PacketType, U: PacketType> From<T> for PacketContext<T, U> {
    fn from(value: T) -> Self {
        Self { time: Utc::now(), id: Uuid::new_v4(), state: PacketState::Received, input_packet: value, output_packet: U::empty() }
    }
}

