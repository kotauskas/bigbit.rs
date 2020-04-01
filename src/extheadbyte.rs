//! The Extended Head Byte format, capable of storing arbitrarily large signed integers and decimal fractions.
//!
//! This format is strictly equivalent to Head Byte if there are 15 or less follow-up (1 exponent + 14 coefficient) bytes (the only exception). Otherwise, both the exponent and the **number** of coefficient bytes are stored using the Linked Bytes format.