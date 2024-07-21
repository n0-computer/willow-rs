#![no_main]

use earthstar::identity_id::IdentityIdentifier as IdentityId;
use earthstar::namespace_id::NamespaceIdentifier as EsNamespaceId;
use libfuzzer_sys::fuzz_target;
use willow_data_model::entry::Entry;
use willow_data_model::grouping::area::Area;
use willow_data_model_fuzz::encode::relative_encoding_random;
use willow_data_model_fuzz::placeholder_params::FakePayloadDigest;

fuzz_target!(
    |data: (&[u8], (EsNamespaceId, Area<16, 16, 16, IdentityId>))| {
        // fuzzed code goes here
        let (random_bytes, namespaced_area) = data;

        smol::block_on(async {
            relative_encoding_random::<
                (EsNamespaceId, Area<16, 16, 16, IdentityId>),
                Entry<16, 16, 16, EsNamespaceId, IdentityId, FakePayloadDigest>,
            >(namespaced_area, random_bytes)
            .await;
        });
    }
);
