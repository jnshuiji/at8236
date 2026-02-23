#![no_std]

use embedded_hal::pwm::SetDutyCycle;

pub struct At8236<P1, P2> {
    in1: P1,
    in2: P2,
    decay_mode: DecayMode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecayMode {
    /// Fast Decay
    Fast,
    /// Slow Decay
    Slow,
}

#[derive(Debug)]
pub enum Error<E> {
    Pwm(E),
}

impl<P1, P2> At8236<P1, P2>
where
    P1: SetDutyCycle,
    P2: SetDutyCycle<Error = P1::Error>,
{
    pub fn new(in1: P1, in2: P2) -> Self {
        // Ensure the maximum duty cycle (resolution/frequency) of both PWM channels is the same
        // If not, the calculation logic will be incorrect.
        assert_eq!(in1.max_duty_cycle(), in2.max_duty_cycle());

        Self {
            in1,
            in2,
            decay_mode: DecayMode::Fast,
        }
    }

    #[inline]
    pub fn set_decay_mode(&mut self, mode: DecayMode) {
        self.decay_mode = mode;
    }

    /// Get the maximum duty cycle value (PWM resolution)
    #[inline]
    pub fn max_duty_cycle(&self) -> u16 {
        self.in1.max_duty_cycle()
    }

    /// Convert a percentage (0 ~ 100) to a duty cycle value
    #[inline]
    fn percent_to_duty(&self, percent: u8) -> u16 {
        let clamped = percent.clamp(0, 100) as u32;
        let max = self.in1.max_duty_cycle() as u32;
        (max * clamped / 100) as u16
    }

    #[inline]
    pub fn stop(&mut self) -> Result<(), Error<P1::Error>> {
        self.in1.set_duty_cycle_fully_off().map_err(Error::Pwm)?;
        self.in2.set_duty_cycle_fully_off().map_err(Error::Pwm)?;
        Ok(())
    }

    #[inline]
    pub fn brake(&mut self) -> Result<(), Error<P1::Error>> {
        self.in1.set_duty_cycle_fully_on().map_err(Error::Pwm)?;
        self.in2.set_duty_cycle_fully_on().map_err(Error::Pwm)?;
        Ok(())
    }

    /// Forward with raw duty cycle value (0 ~ `max_duty_cycle()`)
    ///
    /// Provides the highest precision control, using the full PWM resolution directly.
    #[inline]
    pub fn forward_duty(&mut self, duty: u16) -> Result<(), Error<P1::Error>> {
        let max_duty = self.in1.max_duty_cycle();
        let duty = duty.min(max_duty);

        match self.decay_mode {
            DecayMode::Fast => {
                self.in1.set_duty_cycle(duty).map_err(Error::Pwm)?;
                self.in2.set_duty_cycle_fully_off().map_err(Error::Pwm)?;
            }
            DecayMode::Slow => {
                self.in1.set_duty_cycle_fully_on().map_err(Error::Pwm)?;
                self.in2
                    .set_duty_cycle(max_duty - duty)
                    .map_err(Error::Pwm)?;
            }
        }
        Ok(())
    }

    /// Reverse with raw duty cycle value (0 ~ `max_duty_cycle()`)
    ///
    /// Provides the highest precision control, using the full PWM resolution directly.
    #[inline]
    pub fn reverse_duty(&mut self, duty: u16) -> Result<(), Error<P1::Error>> {
        let max_duty = self.in1.max_duty_cycle();
        let duty = duty.min(max_duty);

        match self.decay_mode {
            DecayMode::Fast => {
                self.in1.set_duty_cycle_fully_off().map_err(Error::Pwm)?;
                self.in2.set_duty_cycle(duty).map_err(Error::Pwm)?;
            }
            DecayMode::Slow => {
                self.in1
                    .set_duty_cycle(max_duty - duty)
                    .map_err(Error::Pwm)?;
                self.in2.set_duty_cycle_fully_on().map_err(Error::Pwm)?;
            }
        }
        Ok(())
    }

    /// Forward with integer percentage (0 ~ 100)
    #[inline]
    pub fn forward(&mut self, percent: u8) -> Result<(), Error<P1::Error>> {
        self.forward_duty(self.percent_to_duty(percent))
    }

    /// Reverse with integer percentage (0 ~ 100)
    #[inline]
    pub fn reverse(&mut self, percent: u8) -> Result<(), Error<P1::Error>> {
        self.reverse_duty(self.percent_to_duty(percent))
    }
}
