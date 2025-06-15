use component::{back_btn::BackButton, title::TitleText};
use leptos::prelude::*;
use leptos_meta::*;
use state::app_state::AppState;

fn terms_section<T: IntoView>(title: &str, content: T) -> impl IntoView + use<'_, T> {
    view! {
        <div class="term-section mb-6">
            <div class="term-title text-sm font-semibold mb-3">{title}</div>
            <div class="term-content">{content}</div>
        </div>
    }
}

fn bullet_list(items: Vec<&str>) -> impl IntoView + '_ {
    let list_items = items
        .into_iter()
        .map(|item| {
            view! { <li class="mb-2">{item}</li> }
        })
        .collect_view();

    view! { <ul class="list-disc pl-6 py-2 text-xs">{list_items}</ul> }
}

fn text_paragraph(content: &str) -> impl IntoView + use<'_> {
    view! { <p class="mb-3 text-xs">{content}</p> }
}

#[component]
pub fn TermsIos() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - iOS Terms of Service";

    // Define content sections for easier editing
    let intro_content = "Welcome to Yral, a community-driven platform where users share and discover short videos.\n\nThese Terms of Use (\"Terms\") govern your use of the Yral mobile and web application (\"App\", \"Services\"), operated by HotorNot (HON) Gmbh (\"Company\", \"we,\" \"our,\" or \"us\"). By using the App, you agree to these Terms. If you do not agree, please do not use the App.";

    let account_text = "You must be at least 13 years old to use the App and you must be represented by a legal guardian if you are below the legal age in your respective jurisdiction to register or use the App. You represent and warrant that you will provide accurate and up to date information while creating an account with us and you agree to keep the information accurate at all times. You must keep your account password confidential. If at any time, if you fail to comply with the provision of these terms or if activities occur on your account which might cause damage to the Services or infringe or violate any third party rights or violate any laws or regulations, or for any other reason, we reserve the right to, at our discretion, disable your account and remove or disable any content you upload or share. You agree that:";

    let account_bullets = vec![
        "You are at least 13 years old or above the minimum legal age in your jurisdiction (whichever is higher) to use the App. If you are below the legal age in your respective jurisdiction, you must be represented by a legal guardian even if you are over 13 years of age. If you are below 13 years of age, please do not use the App.",
        "You must not be prohibited from receiving any aspect of our service under applicable laws or engaging in payments related services if you are on an applicable denied party listing.",
        "We have not previously disabled your account for violation of law or any of our policies.",
        "You are not a convicted sex offender.",
        "You have read, understood and agreed to be bound by these Terms, privacy policy and community guidelines of the App for yourself and any minor in your care who has access to our Services.",
        "You have the right, authority and legal capacity to agree to these Terms, privacy policy and community guidelines on behalf of yourself. Only persons who can form legally binding contracts under the law of their jurisdiction or those persons (such as minors) that are represented by persons (such as legal guardians) who can form legally binding contracts under the law of their jurisdiction, are permitted to use our Services.",
        "You shall be solely responsible (to us and to others) for the activity on your account and its consequences.",
        "You may not create an account if you are legally prohibited from using our services under local or international laws."
    ];

    view! {
        <Title text=page_title />

        <div class="w-screen min-h-screen bg-black pt-4 pb-12 text-white flex flex-col items-center">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/menu".to_string() />
                    <span class="font-bold">iOS Terms of Service</span>
                    <div></div>
                </div>
            </TitleText>

            <div class="px-8 flex h-full w-full flex-col space-y-8 overflow-hidden overflow-y-auto py-16">
                <div class="text-center mb-6">
                    <h1 class="text-xl font-bold mb-2">Terms of Use | Yral</h1>
                    <div class="text-sm opacity-80 mb-4">
                        <p>
                            <strong>Effective Date:</strong>
                            13th July 2023
                        </p>
                        <p>
                            <strong>Last Updated:</strong>
                            11th April 2025
                        </p>
                    </div>
                </div>

                <div class="text-xs whitespace-pre-line mb-6">{intro_content}</div>

                {terms_section(
                    "1. Your Account & Registration",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">{account_text}</p>
                            {bullet_list(account_bullets)}
                        </div>
                    },
                )}

                {terms_section(
                    "2. Community Guidelines on Yral Content",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                "Yral is a platform for user-generated video content. By posting any content on or through our Services, you hereby grant us a non-exclusive, fully paid and royalty-free, worldwide, limited licence to display such content as felt appropriate, in any media formats through any media channels, and delete such content solely for the purpose of operating and improving the Yral platform. We do not claim any ownership or modification or derivative rights in any Content or to the underlying works in the content that you post on or through our Services. You agree that your use of the Services does not grant you any right to any compensation or share in revenue or value. To the extent it's necessary, when you generate content, you also grant us, our affiliates, and our business partners the unrestricted, worldwide, perpetual right and licence to use your name, likeness, and voice. You will not be entitled to any compensation from us, our affiliates, or our business partners if your name, likeness, or voice is conveyed through the Services. You represent, warrant and covenant, as applicable, that:"
                            </p>

                            {bullet_list(
                                vec![
                                    "You own the content posted by you on or through our Services or otherwise have the right to grant the licence set forth in this section.",
                                    "The posting and use of your content on or through our Services does not violate the privacy rights, copyrights, contractual rights, intellectual property rights or any other rights of any person",
                                    "The posting of your content does not result in a breach of contract between you and a third party",
                                    "Your actions on Yral shall not be in violation of applicable law or regulation. We are not responsible for any Content posted by you or any consequences thereof.",
                                ],
                            )}

                            <p class="mb-3 text-xs">
                                "You agree that you shall not host, display, upload, modify, publish, transmit, store, update or share any information that: "
                            </p>

                            {bullet_list(
                                vec![
                                    "Belongs to another person and to which you do not have any right to;",
                                    "Is grossly harmful, harassing, blasphemous, defamatory, obscene, pornographic, paedophilic, libellous, invasive of another's privacy, including bodily privacy, insulting or harassing on the basis of gender, libellous, hateful, racially or ethnically objectionable, disparaging, relating or encouraging money laundering or gambling, or otherwise unlawful inconsistent with or contrary to the laws in force in any manner whatever;",
                                    "Harms minors in any way;",
                                    "Infringes any patent, trademark, copyright or other proprietary rights;",
                                    "Violates any law for the time being in force;",
                                    "Deceives or misleads the addressee about the origin of such messages or communicates any information which is grossly offensive or menacing in nature or which is patently false or misleading in nature but may reasonably be perceived as a fact;",
                                    "Impersonates another person;",
                                    "Contains software viruses or any other computer code, files or programs designed to interrupt, destroy or limit the functionality of any computer resource;",
                                    "Intimidates or harasses another, or promotes sexually explicit material, violence or discrimination based on race, sex, religion, nationality, disability, sexual orientation or age.",
                                    "Contains material that contains a threat of any kind, including threats of physical violence;",
                                    "Is slanderous or defamatory;",
                                    "Is patently false and untrue, and is written or published in any form, with the intent to mislead or harass a person, entity or agency for financial gain or to cause any injury to any person.",
                                ],
                            )}

                            <p class="mb-3 text-xs">
                                You acknowledge and agree that the content uploaded is the sole responsibility of the user who has uploaded such content and the views expressed are their own. We are not responsible for the content posted by you. You shall be solely responsible for the content posted, shared, modified, uploaded, transmitted, updated and hosted by you. You agree that your use of our Services will conform to the community guidelines as defined by the App. You agree that in case of non-compliance with applicable laws, or with these terms and conditions, or with the Yral privacy policy or with our community guidelines, we have the right to terminate your access or usage rights to Yral immediately and remove non-compliant content from Yral. You hereby agree that you will never use Yral in violation of any applicable law. If any violation of these terms is brought to our actual knowledge by an affected person, we shall act within twenty-four hours and where applicable, work with the user or owner of such information to delete/disable such information/content that is in contravention of these terms. We will not intimate you if any of the content posted by you is taken down / deleted or your account is disabled pursuant to the above. We will preserve such information and associated records for at least one hundred and eighty days for investigation purposes. Please contact the grievance officer whose details are provided in the last section of these terms, in case of any violation or grievances.
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "3. Community Guidelines",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                "You agree to follow Yral's Community Guidelines, which prohibit any sort of objectionable content that is:"
                            </p>

                            {bullet_list(
                                vec![
                                    "Defamatory, discriminatory, or mean-spirited content, including references or commentary about religion, race, sexual orientation, gender, national/ethnic origin, or other targeted groups—particularly if the app could humiliate, intimidate, or harm a targeted individual or group. Professional political satirists and humorists are exempt from this requirement.",
                                    "Realistic portrayals of people or animals being killed, maimed, tortured, or abused, or content that encourages violence. In games, \"enemies\" cannot solely target a specific race, culture, real government, corporation, or any other real entity.",
                                    "Depictions that encourage illegal or reckless use of weapons and dangerous objects, or facilitate the purchase of firearms or ammunition.",
                                    "Overtly sexual or pornographic material—defined as \"explicit descriptions or displays of sexual organs or activities intended to stimulate erotic rather than aesthetic or emotional feelings.\" This includes \"hookup\" apps and other applications that may contain pornography or be used to facilitate prostitution, human trafficking, or exploitation.",
                                    "Inflammatory religious commentary or inaccurate or misleading quotations of religious texts.",
                                    "False information and features, including inaccurate device data or trick/joke functionality, such as fake location trackers. Claiming an app is \"for entertainment purposes\" will not exempt it from this guideline. Apps enabling anonymous or prank phone calls or SMS/MMS messaging will be rejected.",
                                    "Harmful content that attempts to profit from recent or current events such as violent conflicts, terrorist attacks, or epidemics.",
                                ],
                            )}

                            <p class="mb-3 text-xs">
                                Violating these guidelines will result in content removal, account restrictions, or permanent suspension.
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "4. Content Moderation & Safety",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                Yral uses a mix of AI enabled automated filters and human moderation to detect and manage objectionable content. Our automated filters automatically delete Not Safe For Work (NSFW) content containing nudity, pornography, or other harmful content that violates our community guidelines. Further, users have the ability to report NSFW content while using the App. Users are encouraged to report NSFW content that our automated filters miss. Such reports are prioritized by our moderation team.
                            </p>

                            <p class="mb-3 text-xs">
                                We have a dedicated team to review all valid reports within 24 hours and take appropriate action, including:
                            </p>

                            {bullet_list(
                                vec![
                                    "Removing the reported content",
                                    "Temporarily or permanently banning the offending user",
                                ],
                            )}

                            <p class="mb-3 text-xs">
                                To ensure platform integrity and a safe environment, Yral takes violations of community standards very seriously and will block abusive users immediately and indefinitely. We aim to be transparent with moderation decisions and provide users with feedback when appropriate.
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "5. NSFW Content Policy",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                By enabling the NSFW (Not Safe for Work) content toggle, you acknowledge and consent to viewing adult-oriented content within your feed. This content may include themes that are intended for mature audiences. You agree that:
                            </p>

                            {bullet_list(
                                vec![
                                    "You are of legal age to view such content as per the laws and regulations of your jurisdiction.",
                                    "The platform is not responsible for any distress or offense caused by NSFW content.",
                                    "You may disable NSFW content at any time by turning off the toggle in your settings.",
                                ],
                            )}
                        </div>
                    },
                )}

                {terms_section(
                    "6. Blocking Other Users",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                You may block, report, or both block and report other users at any time if you experience inconvenience, discomfort, or simply wish to do so. When blocked:
                            </p>

                            {bullet_list(
                                vec![
                                    "Their videos will no longer appear in your feed",
                                    "They cannot view or interact with your content",
                                    "They are not notified of the block",
                                ],
                            )}

                            <p class="mb-3 text-xs">
                                This is a key part of maintaining your control over your in-app experience.
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "7. Use License (EULA Equivalent)",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                We grant you a limited, non-exclusive, non-transferable, revocable license to use the Yral mobile and web application (the "App") solely for your personal, non-commercial use on any Apple-branded device that you own or control, and as permitted by the Usage Rules set forth in the Apple App Store Terms of Service. These Usage Rules govern how apps can be used on Apple devices, including limitations on sharing, duplication, and access based on your Apple ID and device ownership.
                            </p>

                            <p class="mb-3 text-xs">You agree not to:</p>

                            <p class="mb-2 text-xs font-medium">Technical Restrictions</p>
                            {bullet_list(
                                vec![
                                    "Reverse-engineer, decompile, disassemble, modify, translate, or attempt to discover the source code or underlying ideas or algorithms of the App;",
                                    "Interfere with or disrupt the App, its servers, or networks connected to the App;",
                                    "Upload, post, or transmit any viruses, worms, defects, Trojan horses, or any malicious code through the App;",
                                ],
                            )}

                            <p class="mb-2 text-xs font-medium">Commercial Misuse</p>
                            {bullet_list(
                                vec![
                                    "Reproduce, redistribute, sublicense, lease, rent, publish, sell, or exploit the App or any portion of the App in any way;",
                                ],
                            )}

                            <p class="mb-2 text-xs font-medium">User Data and Privacy</p>
                            {bullet_list(
                                vec![
                                    "Access or attempt to access data of other users without authorization.",
                                ],
                            )}

                            <p class="mb-3 text-xs">You acknowledge and agree that:</p>
                            {bullet_list(
                                vec![
                                    "The license is granted by HotorNot (HON) GmbH (\"we\", \"our\", or \"us\"), and not by Apple.",
                                    "We are solely responsible for the App and the content moderation therein.",
                                    "Apple has no obligation whatsoever to furnish any maintenance or support services with respect to the App.",
                                    "In the event of any failure of the App to conform to any applicable warranty, you may notify Apple, and Apple will refund the purchase price (if any). To the maximum extent permitted by law, Apple will have no other warranty obligation with respect to the App.",
                                    "We are solely responsible for addressing any claims by you or any third party relating to the App or your possession and/or use of the App, including but not limited to: product liability claims, any claim that the App fails to conform to any applicable legal or regulatory requirement, and claims arising under consumer protection, privacy, or similar legislation.",
                                    "If a third party claims that the App or your possession and use of the App infringes their intellectual property rights, you acknowledge that HotorNot (HON) GmbH, not Apple, is solely responsible for addressing, investigating, defending against, settling, and resolving such claims.",
                                    "Apple and its subsidiaries are third-party beneficiaries of this EULA. Upon your acceptance of this agreement, Apple will have the right (and will be deemed to have accepted the right) to enforce these Terms against you as a third-party beneficiary.",
                                ],
                            )}

                            <p class="mb-3 text-xs">
                                "This license is effective until terminated by you or by us. You may terminate the license at any time by uninstalling the App or deleting your account through the App's settings. Your rights under this license will also terminate automatically without notice if you fail to comply with any of its terms. Upon termination, you must cease all use of the App and destroy all copies, full or partial, of the App."
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "8. Content Removal and Disabling or Terminating Your Account",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                We may remove any content, suspend or terminate your account without notice if:
                            </p>
                            {bullet_list(
                                vec![
                                    "You breach these Terms",
                                    "You violate laws or community accepted standards",
                                    "Your behaviour puts other users at risk",
                                ],
                            )}

                            <p class="mb-3 text-xs">
                                "You may delete your account at any time via the app's settings."
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "9. Privacy Policy",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                "We take your privacy seriously. Please refer to "
                                <a href="/privacy-policy" class="underline text-blue-400">
                                    Privacy Policy
                                </a>
                                "for detailed information on the data we collect, how we use it, your rights, and how to exercise them. You can access the full policy from the menu."
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "10. Disclaimers",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                "To the maximum extent permitted by applicable law, the App is licensed 'as is,' and you use it at your sole risk. The developer disclaims all warranties, express or implied. We do not guarantee:"
                            </p>

                            {bullet_list(
                                vec![
                                    "The accuracy or reliability of content. All opinions expressed belong solely to the users",
                                    "Uninterrupted or error-free service.",
                                    "That content will remain available or accessible.",
                                ],
                            )}

                            <p class="mb-3 text-xs">Use Yral at your own risk.</p>
                        </div>
                    },
                )}

                {terms_section(
                    "11. Limitation of Liability",
                    view! {
                        <div>
                            <p class="mb-3 text-xs">
                                "We shall not be liable to you for any loss or damages or claims including but not limited to the following:"
                            </p>

                            {bullet_list(
                                vec![
                                    "Any loss of profit, opportunity or goodwill;",
                                    "Any loss of data;",
                                    "Any damage incurred as a result of your reliance on any advertisement appearing on the Service;",
                                    "Any damage incurred as a result of any changes to the Service, its features and any temporary or permanent termination of our Services;",
                                    "Any damage incurred as a result of your failure to provide us with accurate information and your failure to keep your account details and password confidential and secure;",
                                    "Any damage incurred as a result of the actions of another user.",
                                ],
                            )}

                            <p class="mb-3 text-xs ">
                                Any dispute that may arise between you and any third party arising from your use of the services shall be only between you and a third party and you release us and our affiliates from any such claims and damages connected with such disputes.
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "12. Changes to These Terms ",
                    view! {
                        <div>
                            <p class="mb-3 text-xs ">
                                "We may update these Terms from time to time. We'll notify you of major changes through the App or email. Continued use of the App after changes means you accept the revised Terms."
                            </p>
                        </div>
                    },
                )}

                {terms_section(
                    "13. Contact Us",
                    view! {
                        <div>
                            {text_paragraph("Have questions or concerns?")}
                            {text_paragraph("📧 Email us at: support@yral.com")}
                        </div>
                    },
                )}
            </div>
        </div>
    }
}
