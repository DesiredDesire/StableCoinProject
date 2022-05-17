pub use crate::traits::pausing::*;
use brush::{contracts::ownable::*, contracts::pausable::*, modifiers};

impl<T: PausableStorage + OwnableStorage> Pausing for T {
    #[modifiers(only_owner)]
    #[modifiers(when_not_paused)]
    default fn pause(&mut self) -> Result<(), PausingError> {
        PausableStorage::get_mut(self).paused = true;
        self._emit_paused_event(Self::env().caller());
        Ok(())
    }

    #[modifiers(only_owner)]
    #[modifiers(when_paused)]
    default fn unpause(&mut self) -> Result<(), PausingError> {
        PausableStorage::get_mut(self).paused = false;
        self._emit_unpaused_event(Self::env().caller());
        Ok(())
    }
}
