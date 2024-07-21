#![no_main]

use earthstar::identity_id::IdentityIdentifier as IdentityId;
use earthstar::namespace_id::NamespaceIdentifier as EsNamespaceId;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use ufotofu::local_nb::consumer::TestConsumer;
use ufotofu::local_nb::{BulkConsumer, BulkProducer};
use willow_data_model::encoding::error::{DecodeError, EncodingConsumerError};
use willow_data_model::encoding::parameters::{Decodable, Encodable};
use willow_data_model::entry::Entry;
use willow_data_model::parameters::PayloadDigest;
use willow_data_model_fuzz::encode::encoding_roundtrip;

fuzz_target!(|data: (
    Entry<3, 3, 3, EsNamespaceId, IdentityId, FakePayloadDigest>,
    TestConsumer<u8, u16, ()>
)| {
    let (entry, mut consumer) = data;

    smol::block_on(async {
        encoding_roundtrip::<_, TestConsumer<u8, u16, ()>>(entry, &mut consumer).await;
    });
});
