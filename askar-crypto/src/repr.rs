//! Traits for exposing key data representations

#[cfg(feature = "alloc")]
use crate::buffer::SecretBytes;
use crate::{
    buffer::WriteBuffer,
    error::Error,
    generic_array::{typenum::Unsigned, ArrayLength},
};

/// A seed used in key generation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seed<'d> {
    /// A seed byte string with a selected generation method
    Bytes(&'d [u8], SeedMethod),
}

impl<'d> From<&'d [u8]> for Seed<'d> {
    fn from(seed: &'d [u8]) -> Self {
        Self::Bytes(seed, SeedMethod::Preferred)
    }
}

/// Supported deterministic key methods
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedMethod {
    /// Use the preferred method for the current key algorithm
    Preferred,
    /// Generate a BLS key according to bls-signatures-draft-04
    BlsKeyGenDraft4,
    /// Random bytes compatible with libsodium's randombytes_buf_deterministic.
    /// The seed must be 32 bytes in length
    RandomDet,
}

/// Key generation operations
pub trait KeyGen {
    /// Generate a new random key.
    fn generate() -> Result<Self, Error>
    where
        Self: Sized;

    /// Generate a new deterministic key.
    fn from_seed(_seed: Seed<'_>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        return Err(err_msg!(
            Unsupported,
            "Key generation from seed not supported"
        ));
    }
}

/// Convert between key instance and key secret bytes
pub trait KeySecretBytes: KeyMeta {
    /// Create a new key instance from a slice of key secret bytes.
    fn from_secret_bytes(key: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Access a temporary slice of the key secret bytes, if any.
    fn with_secret_bytes<O>(&self, f: impl FnOnce(Option<&[u8]>) -> O) -> O;
}

/// Object-safe trait for exporting key secret bytes
pub trait ToSecretBytes {
    /// Get the length of a secret key
    fn secret_bytes_length(&self) -> Result<usize, Error>;

    /// Write the key secret bytes to a buffer.
    fn write_secret_bytes(&self, out: &mut dyn WriteBuffer) -> Result<(), Error>;

    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    /// Write the key secret bytes to a new allocated buffer.
    fn to_secret_bytes(&self) -> Result<SecretBytes, Error> {
        let mut buf = SecretBytes::with_capacity(128);
        self.write_secret_bytes(&mut buf)?;
        Ok(buf)
    }
}

impl<K> ToSecretBytes for K
where
    K: KeySecretBytes,
{
    fn secret_bytes_length(&self) -> Result<usize, Error> {
        Ok(<Self as KeyMeta>::KeySize::USIZE)
    }

    fn write_secret_bytes(&self, out: &mut dyn WriteBuffer) -> Result<(), Error> {
        self.with_secret_bytes(|buf| {
            if let Some(buf) = buf {
                out.buffer_write(buf)
            } else {
                Err(err_msg!(MissingSecretKey))
            }
        })
    }
}

/// Convert between key instance and key public bytes.
pub trait KeyPublicBytes: KeypairMeta {
    /// Create a new key instance from a slice of public key bytes.
    fn from_public_bytes(key: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Access a temporary slice of the key public bytes.
    fn with_public_bytes<O>(&self, f: impl FnOnce(&[u8]) -> O) -> O;
}

/// Object-safe trait for exporting key public bytes
pub trait ToPublicBytes {
    /// Get the length of a public key
    fn public_bytes_length(&self) -> Result<usize, Error>;

    /// Write the key public bytes to a buffer.
    fn write_public_bytes(&self, out: &mut dyn WriteBuffer) -> Result<(), Error>;

    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    /// Write the key public bytes to a new allocated buffer.
    fn to_public_bytes(&self) -> Result<SecretBytes, Error> {
        let mut buf = SecretBytes::with_capacity(128);
        self.write_public_bytes(&mut buf)?;
        Ok(buf)
    }
}

impl<K> ToPublicBytes for K
where
    K: KeyPublicBytes,
{
    fn public_bytes_length(&self) -> Result<usize, Error> {
        Ok(<Self as KeypairMeta>::PublicKeySize::USIZE)
    }

    fn write_public_bytes(&self, out: &mut dyn WriteBuffer) -> Result<(), Error> {
        self.with_public_bytes(|buf| out.buffer_write(buf))
    }
}

/// Convert between keypair instance and keypair (secret and public) bytes
pub trait KeypairBytes {
    /// Create a new key instance from a slice of keypair bytes.
    fn from_keypair_bytes(key: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Create a new key instance from a slice of keypair bytes.
    fn with_keypair_bytes<O>(&self, f: impl FnOnce(Option<&[u8]>) -> O) -> O;

    /// Write the keypair bytes to a buffer.
    fn to_keypair_bytes_buffer<B: WriteBuffer>(&self, out: &mut B) -> Result<(), Error> {
        self.with_keypair_bytes(|buf| {
            if let Some(buf) = buf {
                out.buffer_write(buf)
            } else {
                Err(err_msg!(MissingSecretKey))
            }
        })
    }

    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    /// Write the keypair bytes to a new allocated buffer.
    fn to_keypair_bytes(&self) -> Result<SecretBytes, Error> {
        let mut buf = SecretBytes::with_capacity(128);
        self.to_keypair_bytes_buffer(&mut buf)?;
        Ok(buf)
    }
}

/// For concrete secret key types
pub trait KeyMeta {
    /// The size of the key secret bytes
    type KeySize: ArrayLength<u8>;
}

/// For concrete secret + public key types
pub trait KeypairMeta: KeyMeta {
    /// The size of the key public bytes
    type PublicKeySize: ArrayLength<u8>;
    /// The size of the secret bytes and public bytes combined
    type KeypairSize: ArrayLength<u8>;
}
