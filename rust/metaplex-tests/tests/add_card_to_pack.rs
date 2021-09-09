mod utils;

use metaplex_nft_packs::{
    instruction::{AddCardToPackArgs, InitPackSetArgs},
    state::{AccountType, DistributionType},
};
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::TransactionError, transport::TransportError,
};
use utils::*;

async fn setup() -> (
    ProgramTestContext,
    TestPackSet,
    TestMetadata,
    TestMasterEditionV2,
    User,
) {
    let mut context = nft_packs_program_test().start_with_context().await;

    let test_pack_set = TestPackSet::new();
    test_pack_set
        .init(
            &mut context,
            InitPackSetArgs {
                name: [7; 32],
                total_packs: 5,
                mutable: true,
            },
        )
        .await
        .unwrap();

    let test_metadata = TestMetadata::new();
    let test_master_edition = TestMasterEditionV2::new(&test_metadata);

    let user_token_acc = Keypair::new();
    let user = User {
        owner: Keypair::new(),
        token_account: user_token_acc.pubkey(),
    };

    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            &user_token_acc,
            &test_pack_set.authority.pubkey(),
        )
        .await
        .unwrap();

    test_master_edition
        .create(&mut context, Some(10))
        .await
        .unwrap();

    (
        context,
        test_pack_set,
        test_metadata,
        test_master_edition,
        user,
    )
}

#[tokio::test]
async fn success() {
    let (mut context, test_pack_set, test_metadata, test_master_edition, user) = setup().await;

    let test_pack_card = TestPackCard::new(&test_pack_set, 1);
    test_pack_set
        .add_card(
            &mut context,
            &test_pack_card,
            &test_master_edition,
            &test_metadata,
            &user,
            AddCardToPackArgs {
                max_supply: Some(5),
                probability_type: DistributionType::ProbabilityBased,
                probability: 1000000,
                index: test_pack_card.index,
            },
        )
        .await
        .unwrap();

    let pack_card = test_pack_card.get_data(&mut context).await;

    assert_eq!(pack_card.account_type, AccountType::PackCard);
}

#[tokio::test]
async fn fail_invalid_index() {
    let (mut context, test_pack_set, test_metadata, test_master_edition, user) = setup().await;

    let test_pack_card = TestPackCard::new(&test_pack_set, 1);
    test_pack_set
        .add_card(
            &mut context,
            &test_pack_card,
            &test_master_edition,
            &test_metadata,
            &user,
            AddCardToPackArgs {
                max_supply: Some(5),
                probability_type: DistributionType::ProbabilityBased,
                probability: 1000000,
                index: test_pack_card.index,
            },
        )
        .await
        .unwrap();

    context.warp_to_slot(3).unwrap();

    let test_pack_card = TestPackCard::new(&test_pack_set, 1);
    let result = test_pack_set
        .add_card(
            &mut context,
            &test_pack_card,
            &test_master_edition,
            &test_metadata,
            &user,
            AddCardToPackArgs {
                max_supply: Some(5),
                probability_type: DistributionType::ProbabilityBased,
                probability: 1000000,
                index: test_pack_card.index,
            },
        )
        .await;

    assert_transport_error!(
        result.unwrap_err(),
        TransportError::TransactionError(TransactionError::InstructionError(
            1,
            InstructionError::InvalidArgument
        ))
    );
}