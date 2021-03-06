//!
//! Unversioned functions.
//!
//! # Examples
//!
//! ```rust
//! use ruc::*;
//! use serde::{Deserialize, Serialize};
//! use std::{sync::mpsc::channel, thread};
//! use vsdb::{Mapx, MapxOrd, Vecx};
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! struct Animal {
//!     id: u64,
//!     kind: AnimalKind,
//!     owner: AnimalOwner,
//! }
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! enum AnimalKind {
//!     Dog,
//!     Cat,
//!     Pig,
//!     Monkey,
//!     Donkey,
//!     Unknown,
//! }
//!
//! type AnimalID = u64;
//! type AnimalOwner = String;
//!
//! #[derive(Default, Debug, Deserialize, Serialize)]
//! struct AnimalHospital {
//!     owners: Mapx<AnimalOwner, MapxOrd<AnimalID, Animal>>,
//!     discharge_history: Vecx<AnimalID>,
//!     id_alloctor: u64,
//! }
//!
//! impl AnimalHospital {
//!     fn hospitalized(
//!         &mut self,
//!         kind: AnimalKind,
//!         owner: AnimalOwner,
//!     ) -> Result<AnimalID> {
//!         if matches!(kind, AnimalKind::Unknown) {
//!             return Err(eg!("unsupported animal"));
//!         }
//!
//!         let new_id = self.alloc_id();
//!         let animal = Animal {
//!             id: new_id,
//!             kind,
//!             owner: owner.clone(),
//!         };
//!         self.owners
//!             .entry(owner)
//!             .or_insert(MapxOrd::new())
//!             .insert(new_id, animal);
//!
//!         Ok(new_id)
//!     }
//!
//!     fn discharged(&mut self, owner: AnimalOwner, id: AnimalID) -> Result<()> {
//!         if let Some(mut animals) = self.owners.get_mut(&owner) {
//!             if animals.remove(&id).is_none() {
//!                 return Err(eg!("animal ID not found"));
//!             }
//!             self.discharge_history.push(id);
//!
//!             Ok(())
//!         } else {
//!             Err(eg!("owner not found"))
//!         }
//!     }
//!
//!     fn alloc_id(&mut self) -> AnimalID {
//!         self.id_alloctor += 1;
//!         self.id_alloctor
//!     }
//! }
//!
//! let (sender, receiver) = channel();
//!
//! thread::spawn(move || {
//!     let mut ah = AnimalHospital::default();
//!     let ids = (0..100)
//!         .map(|i| ah.hospitalized(AnimalKind::Cat, format!("owner-{}", i)))
//!         .collect::<Result<Vec<_>>>()
//!         .unwrap();
//!     sender
//!         .send((bcs::to_bytes(&ah).unwrap(), ids))
//!         .unwrap();
//! });
//!
//! let (ah_str, ids) = receiver.recv().unwrap();
//!
//! // will be re-initilized when serializing
//! let mut ah = bcs::from_bytes::<AnimalHospital>(&ah_str).unwrap();
//!
//! (0..100)
//!     .zip(ids.iter())
//!     .map(|(i, id)| ah.discharged(format!("owner-{}", i), *id))
//!     .collect::<Result<Vec<_>>>()
//!     .unwrap();
//! ```

pub mod mapx;
pub mod mapx_ord;
pub mod mapx_ord_rawkey;
pub mod mapx_ord_rawvalue;
pub mod mapx_raw;
pub mod orphan;
pub mod vecx;
pub mod vecx_raw;
