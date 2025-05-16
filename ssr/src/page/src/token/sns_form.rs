use candid::Principal;
use sns_validation::{
    config::{
        Bonus, Distribution, InitialBalances, MaximumVotingPowerBonuses, Neuron, Neurons,
        NnsProposal, Proposals, RewardRate, SnsConfigurationFile, Swap, Token, VestingSchedule,
        Voting,
    },
    humanize::{parse_duration, parse_percentage, parse_tokens},
    pbs::nns_pb,
};
use yral_canisters_common::Canisters;

#[derive(Clone, Debug)]
struct NeuronForm {
    stake: nns_pb::Tokens,
    memo: u64,
    dissolve_delay: nns_pb::Duration,
    vesting_period: nns_pb::Duration,
}

impl Default for NeuronForm {
    fn default() -> Self {
        Self {
            stake: parse_tokens("4_500_000 tokens").unwrap(),
            memo: 0,
            dissolve_delay: parse_duration("2 seconds").unwrap(),
            vesting_period: parse_duration("2 seconds").unwrap(),
        }
    }
}

impl NeuronForm {
    fn into_neuron(self, user_principal: Principal) -> Neuron {
        Neuron {
            principal: user_principal.to_string(),
            stake: self.stake,
            memo: self.memo,
            dissolve_delay: self.dissolve_delay,
            vesting_period: self.vesting_period,
        }
    }
}

#[derive(Clone, Debug)]
struct DistributionForm {
    pub total: nns_pb::Tokens,
    neurons: Vec<NeuronForm>,
    initial_balances: InitialBalances,
}

impl Default for DistributionForm {
    fn default() -> Self {
        Self {
            total: parse_tokens("2_502_005 tokens").unwrap(),
            neurons: vec![
                NeuronForm {
                    stake: parse_tokens("1_250_001 tokens").unwrap(),
                    memo: 0,
                    dissolve_delay: parse_duration("0 seconds").unwrap(),
                    vesting_period: parse_duration("2 seconds").unwrap(),
                },
                NeuronForm {
                    stake: parse_tokens("1_000 tokens").unwrap(),
                    memo: 1,
                    dissolve_delay: parse_duration("2 seconds").unwrap(),
                    vesting_period: parse_duration("2 seconds").unwrap(),
                },
            ],
            initial_balances: InitialBalances {
                governance: parse_tokens("0 tokens").unwrap(),
                swap: parse_tokens("1_251_004 tokens").unwrap(),
            },
        }
    }
}

impl DistributionForm {
    fn into_distribution(self, user_principal: Principal) -> Distribution {
        Distribution {
            total: self.total,
            // neurons: self
            //     .neurons
            //     .into_iter()
            //     .map(|n| n.into_neuron(user_principal))
            //     .collect(),
            neurons: self
                .neurons
                .into_iter()
                .map(|n| n.into_neuron(user_principal))
                .collect(),
            initial_balances: self.initial_balances,
        }
    }
}

#[derive(Clone)]
pub struct SnsFormState {
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo_b64: Option<String>,
    pub symbol: Option<String>,
    pub transaction_fee: nns_pb::Tokens,
    pub proposals: Proposals,
    pub neurons: Neurons,
    pub voting: Voting,
    distribution: DistributionForm,
    pub swap: Swap,
    pub nns_proposal: NnsProposal,
}

impl Default for SnsFormState {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            logo_b64: None,
            symbol: None,
            transaction_fee: parse_tokens("1 e8s").unwrap(),
            proposals: Proposals {
                rejection_fee: parse_tokens("1 token").unwrap(),
                initial_voting_period: parse_duration("86400 seconds").unwrap(),
                maximum_wait_for_quiet_deadline_extension: parse_duration("2 seconds").unwrap(),
            },
            neurons: Neurons {
                minimum_creation_stake: parse_tokens("2 e8s").unwrap(),
            },
            voting: Voting {
                minimum_dissolve_delay: parse_duration("2 seconds").unwrap(),
                maximum_voting_power_bonuses: MaximumVotingPowerBonuses {
                    dissolve_delay: Bonus {
                        duration: parse_duration("8 years").unwrap(),
                        bonus: parse_percentage("100%").unwrap(),
                    },
                    age: Bonus {
                        duration: parse_duration("4 years").unwrap(),
                        bonus: parse_percentage("25%").unwrap(),
                    },
                },
                reward_rate: RewardRate {
                    initial: parse_percentage("10%").unwrap(),
                    r#final: parse_percentage("2.25%").unwrap(),
                    transition_duration: parse_duration("12 years").unwrap(),
                },
            },
            distribution: DistributionForm::default(),
            swap: Swap {
                minimum_participants: 1,
                minimum_direct_participation_icp: Some(parse_tokens("15 e8s").unwrap()),
                maximum_direct_participation_icp: Some(parse_tokens("1 tokens").unwrap()),
                minimum_participant_icp: parse_tokens("2000 e8s").unwrap(),
                maximum_participant_icp: parse_tokens("1 tokens").unwrap(),
                duration: parse_duration("5 seconds").unwrap(),
                neurons_fund_participation: Some(false),
                vesting_schedule: VestingSchedule {
                    events: 2,
                    interval: parse_duration("5 seconds").unwrap(),
                },
                minimum_icp: None,
                maximum_icp: None,
                confirmation_text: None,
                restricted_countries: None,
                start_time: Some(nns_pb::GlobalTimeOfDay {
                    seconds_after_utc_midnight: Some(1),
                }),
                neurons_fund_investment_icp: None,
            },
            nns_proposal: NnsProposal {
                title: "Creator DAO Stub".into(),
                url: Some("https://yral.com".into()),
                summary: "Creator DAO Stub".into(),
            },
        }
    }
}

impl SnsFormState {
    pub fn try_update_total_distribution_tokens(&mut self, tokens: nns_pb::Tokens) {
        // let tx_fee = self.transaction_fee.e8s.unwrap();
        let tokens = tokens.e8s.unwrap();
        let total = tokens;
        let swap = tokens / 2;
        let governance = 0_u64;

        let voting_neuron = 2000_u64;
        let non_voting_neuron = total - (swap + voting_neuron + governance);

        self.distribution.total = nns_pb::Tokens { e8s: Some(total) };
        self.distribution.neurons = vec![
            NeuronForm {
                stake: nns_pb::Tokens {
                    e8s: Some(non_voting_neuron),
                },
                memo: 0,
                dissolve_delay: parse_duration("0 seconds").unwrap(),
                vesting_period: parse_duration("2 seconds").unwrap(),
            },
            NeuronForm {
                stake: nns_pb::Tokens {
                    e8s: Some(voting_neuron),
                },
                memo: 1,
                dissolve_delay: parse_duration("2 seconds").unwrap(),
                vesting_period: parse_duration("2 seconds").unwrap(),
            },
        ];
        self.distribution.initial_balances.swap = nns_pb::Tokens { e8s: Some(swap) };
        self.distribution.initial_balances.governance = nns_pb::Tokens {
            e8s: Some(governance),
        };
    }

    pub fn try_into_config(
        self,
        canisters: &Canisters<true>,
    ) -> Result<SnsConfigurationFile, String> {
        let user_principal = canisters.user_principal();
        let user_canister = canisters.user_canister();

        let prt = self.distribution.clone();
        log::info!("total distribution: {prt:?}");

        Ok(SnsConfigurationFile {
            name: self.name.clone().ok_or("Name is required")?,
            description: self.description.ok_or("Description is required")?,
            logo_b64: self.logo_b64.clone().ok_or("Logo is required")?,
            url: format!("https://yral.com/profile/{user_principal}"),
            principals: vec![],
            fallback_controller_principals: vec![
                user_principal.to_string(),
                user_canister.to_string(),
            ],
            dapp_canisters: vec![user_canister.to_string()],
            token: Token {
                name: self.name.ok_or("Name is required")?,
                symbol: self.symbol.ok_or("Symbol is required")?,
                transaction_fee: self.transaction_fee,
                logo_b64: self.logo_b64.ok_or("Logo is required")?,
            },
            proposals: self.proposals,
            neurons: self.neurons,
            voting: self.voting,
            distribution: self.distribution.into_distribution(user_principal),
            swap: self.swap,
            nns_proposal: self.nns_proposal,
        })
    }

    pub fn total_distrubution(&self) -> nns_pb::Tokens {
        self.distribution.total
    }
}
