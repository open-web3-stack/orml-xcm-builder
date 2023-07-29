use frame_support::traits::OriginTrait;
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::TrailingZeroInput;
use sp_std::marker::PhantomData;
use xcm::latest::prelude::*;
use xcm_executor::traits::{ConvertLocation, ConvertOrigin};

/// Tinkernet ParaId used when matching Multisig MultiLocations.
const TINKERNET_PARA_ID: u32 = 2125;

/// Tinkernet Multisig pallet instance used when matching Multisig MultiLocations.
const TINKERNET_MULTISIG_PALLET: u8 = 71;

/// Constant derivation function for Tinkernet Multisigs.
/// Uses the Tinkernet genesis hash as a salt.
pub fn derive_tinkernet_multisig<AccountId: Decode>(id: u128) -> Result<AccountId, ()> {
    AccountId::decode(&mut TrailingZeroInput::new(
        &(
            // The constant salt used to derive Tinkernet Multisigs, this is Tinkernet's genesis hash.
            H256([
                212, 46, 150, 6, 169, 149, 223, 228, 51, 220, 121, 85, 220, 42, 112, 244, 149, 243,
                80, 243, 115, 218, 162, 0, 9, 138, 232, 68, 55, 129, 106, 210,
            ]),
            // The actual multisig integer id.
            u32::try_from(id).map_err(|_| ())?,
        )
            .using_encoded(blake2_256),
    ))
    .map_err(|_| ())
}

/// Convert a Tinkernet Multisig `MultiLocation` value into a local `AccountId`.
pub struct TinkernetMultisigAsAccountId<AccountId>(PhantomData<AccountId>);
impl<AccountId: Decode + Clone> ConvertLocation<AccountId>
    for TinkernetMultisigAsAccountId<AccountId>
{
    fn convert_location(location: &MultiLocation) -> Option<AccountId> {
        match location {
            MultiLocation {
                // Parents will match from the perspective of the relay or one of it's child parachains.
                parents: 0 | 1,
                interior:
                    X3(
                        Parachain(TINKERNET_PARA_ID),
                        PalletInstance(TINKERNET_MULTISIG_PALLET),
                        // Index from which the multisig account is derived.
                        GeneralIndex(id),
                    ),
            } => derive_tinkernet_multisig(*id).ok(),
            _ => None,
        }
    }
}

/// Convert a Tinkernet Multisig `MultiLocation` value into a `Signed` origin.
pub struct TinkernetMultisigAsNativeOrigin<RuntimeOrigin>(PhantomData<RuntimeOrigin>);
impl<RuntimeOrigin: OriginTrait> ConvertOrigin<RuntimeOrigin>
    for TinkernetMultisigAsNativeOrigin<RuntimeOrigin>
where
    RuntimeOrigin::AccountId: Decode,
{
    fn convert_origin(
        origin: impl Into<MultiLocation>,
        kind: OriginKind,
    ) -> Result<RuntimeOrigin, MultiLocation> {
        let origin = origin.into();
        match (kind, origin) {
            (
                OriginKind::Native,
                MultiLocation {
                    // Parents will match from the perspective of the relay or one of it's child parachains.
                    parents: 0 | 1,
                    interior:
                        X3(
                            Junction::Parachain(TINKERNET_PARA_ID),
                            Junction::PalletInstance(TINKERNET_MULTISIG_PALLET),
                            // Index from which the multisig account is derived.
                            Junction::GeneralIndex(id),
                        ),
                },
            ) => Ok(RuntimeOrigin::signed(
                derive_tinkernet_multisig(id).map_err(|_| origin)?,
            )),
            (_, origin) => Err(origin),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::sp_runtime::AccountId32;
    use parity_scale_codec::Encode;
    use xcm::latest::{
        Junction::{GeneralIndex, PalletInstance, Parachain},
        Junctions::X3,
        MultiLocation,
    };

    #[test]
    fn accountid32() {
        assert_eq!(
            TinkernetMultisigAsAccountId::<AccountId32>::convert_location(&MultiLocation {
                parents: 1,
                interior: X3(Parachain(2125), PalletInstance(71), GeneralIndex(0)),
            })
            .unwrap()
            .encode(),
            [
                97, 160, 244, 60, 133, 145, 170, 26, 202, 108, 203, 156, 114, 116, 175, 30, 156,
                195, 43, 101, 243, 51, 193, 162, 152, 188, 30, 165, 244, 81, 70, 90
            ]
        )
    }

    #[test]
    fn accountid20() {
        assert_eq!(
            TinkernetMultisigAsAccountId::<[u8; 20]>::convert_location(&MultiLocation {
                parents: 1,
                interior: X3(Parachain(2125), PalletInstance(71), GeneralIndex(1)),
            })
            .unwrap()
            .encode(),
            [
                243, 9, 113, 183, 227, 177, 166, 120, 142, 144, 184, 186, 80, 212, 65, 22, 152,
                105, 190, 155
            ]
        )
    }
}
