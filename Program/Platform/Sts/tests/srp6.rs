mod common;
use common::{generated, Client};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sts::srp6::{self, ServerExchange};

#[test]
fn independent_client_and_server_round_trip() {
    let mut rng = StdRng::seed_from_u64(0x735250);
    for _ in 0..24 {
        let login = generated(&mut rng);
        let password = generated(&mut rng);
        let record = srp6::enroll(&login, password.as_bytes(), &mut rng);
        let mut random = [0; 32];
        rng.fill_bytes(&mut random);
        let server =
            ServerExchange::start(&login, &record.salt, &record.verifier, &random).unwrap();
        let client = Client::new(
            &login,
            password.as_bytes(),
            &record.salt,
            server.public(),
            &mut rng,
        );
        let verified = server.verify(&client.public, &client.proof).unwrap();
        assert_eq!(verified.proof.as_slice(), client.server_proof);
        assert_eq!(verified.key, client.key);
        assert_eq!(verified.key.len(), 64);
    }
}
#[test]
fn wrong_password_login_and_mutated_proof_fail() {
    let mut rng = StdRng::seed_from_u64(82);
    let login = generated(&mut rng);
    let password = generated(&mut rng);
    let record = srp6::enroll(&login, password.as_bytes(), &mut rng);
    for mode in 0..3 {
        let server =
            ServerExchange::start(&login, &record.salt, &record.verifier, &[5; 32]).unwrap();
        let other = generated(&mut rng);
        let mut client = Client::new(
            if mode == 1 { &other } else { &login },
            if mode == 0 {
                other.as_bytes()
            } else {
                password.as_bytes()
            },
            &record.salt,
            server.public(),
            &mut rng,
        );
        if mode == 2 {
            client.proof[0] ^= 1;
        }
        assert!(server.verify(&client.public, &client.proof).is_err());
    }
}
#[test]
fn zero_mod_n_and_malformed_publics_fail() {
    let mut rng = StdRng::seed_from_u64(83);
    let login = generated(&mut rng);
    let password = generated(&mut rng);
    let record = srp6::enroll(&login, password.as_bytes(), &mut rng);
    for public in [
        vec![],
        vec![0; 4],
        srp6::number_bytes(&srp6::modulus()),
        vec![1; 129],
        vec![1; 3],
    ] {
        let server =
            ServerExchange::start(&login, &record.salt, &record.verifier, &[5; 32]).unwrap();
        assert!(server.verify(&public, &[1; 32]).is_err());
    }
}
#[test]
fn malformed_verifiers_and_zero_ephemeral_fail() {
    let mut rng = StdRng::seed_from_u64(84);
    let login = generated(&mut rng);
    let password = generated(&mut rng);
    let record = srp6::enroll(&login, password.as_bytes(), &mut rng);
    for v in [
        vec![],
        vec![0; 4],
        vec![1; 3],
        srp6::number_bytes(&srp6::modulus()),
    ] {
        assert!(ServerExchange::start(&login, &record.salt, &v, &[5; 32]).is_err());
    }
    assert!(ServerExchange::start(&login, &record.salt, &record.verifier, &[0; 32]).is_err());
    assert!(ServerExchange::start(&login, &[0; 4], &record.verifier, &[5; 32]).is_err());
    assert!(ServerExchange::start(&login, &[1; 3], &record.verifier, &[5; 32]).is_err());
}
#[test]
fn server_rejects_truncated_or_extended_m1() {
    let mut rng = StdRng::seed_from_u64(85);
    let login = generated(&mut rng);
    let password = generated(&mut rng);
    let record = srp6::enroll(&login, password.as_bytes(), &mut rng);
    for size in [0, 31, 33] {
        let server =
            ServerExchange::start(&login, &record.salt, &record.verifier, &[5; 32]).unwrap();
        let client = Client::new(
            &login,
            password.as_bytes(),
            &record.salt,
            server.public(),
            &mut rng,
        );
        let mut proof = client.proof;
        proof.resize(size, 0);
        assert!(server.verify(&client.public, &proof).is_err());
    }
}
