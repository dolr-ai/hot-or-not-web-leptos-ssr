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

    view! {
        <ul class="list-disc pl-6 py-2 text-xs">
            {list_items}
        </ul>
    }
}

fn text_paragraph(content: &str) -> impl IntoView + use<'_> {
    view! {
        <p class={"mb-3 text-xs"}>{content}</p>
    }
}

#[component]
pub fn TermsAndroid() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Android Terms of Service";

    // Define content sections for easier editing
    let intro_content = "Welcome to Yral, a community-driven platform where users share and discover short videos.\n\nThese Terms of Use (\"Terms\") govern your use of the Yral mobile and web application (\"App\", \"Services\"), operated by HotorNot (HON) Gmbh (\"Company\", \"we,\" \"our,\" or \"us\"). By using the App, you agree to these Terms. If you do not agree, please do not use the App.";

    let account_bullets = vec![
        "Provide accurate information and keep it updated.",
        "Not be a convicted sex offender.",
        "Comply with all applicable laws.",
        "Accept full responsibility for activity on your account."
    ];

    let csae_bullets = vec![
        "Strictly prohibit any form of CSAM (Child Sexual Abuse Material), grooming, exploitation, or harmful behavior towards minors.",
        "Use AI-based detection and human moderation to filter and remove such content immediately.",
        "Promptly report violations to appropriate legal authorities and child safety organizations.",
        "Provide clear reporting options for users to flag CSAE-related content, which is reviewed within 24 hours.",
        "Comply with all applicable local and international CSAE-related laws."
    ];

    let community_guidelines_bullets = vec![
        "Defamatory, obscene, or hateful content",
        "Sexual content involving minors or any form of abuse",
        "Content promoting violence or illegal activity",
        "Impersonation, misinformation, or threats",
        "Content with viruses or malicious software"
    ];

    let moderation_bullets = vec![
        "Report inappropriate or harmful content",
        "Expect reported content to be reviewed within 24 hours"
    ];

    let moderation_rights_bullets = vec![
        "Remove content without prior notice",
        "Suspend or terminate accounts in violation"
    ];

    let user_responsibilities_bullets = vec![
        "Follow all rules and applicable laws while using the App",
        "Only post content you have rights to",
        "Not infringe on others' privacy, copyright, or legal protections",
        "Use reporting tools responsibly and not misuse moderation features"
    ];

    let blocking_bullets = vec![
        "Block or report other users",
        "Filter their content experience (e.g., by disabling NSFW content)",
        "Control who interacts with their content and profile"
    ];

    let liability_bullets = vec![
        "Damages due to app errors or downtime",
        "Content uploaded by users",
        "Loss of data or revenue"
    ];

    view! {
        <Title text=page_title />

        <div class="w-screen min-h-screen bg-black pt-4 pb-12 text-white flex flex-col items-center">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/menu".to_string() />
                    <span class="font-bold">Android Terms of Service</span>
                    <div></div>
                </div>
            </TitleText>

            <div class="px-8 flex h-full w-full flex-col space-y-8 overflow-hidden overflow-y-auto py-16">
                <div class="text-center mb-6">
                    <h1 class="text-xl font-bold mb-2">Terms of Use | Yral</h1>
                    <div class="text-sm opacity-80 mb-4">
                        <p><strong>Effective Date:</strong> 13th July 2023</p>
                        <p><strong>Last Updated:</strong> 22nd May 2025</p>
                    </div>
                </div>

                <div class="text-xs whitespace-pre-line mb-6">{intro_content}</div>

                {terms_section("1. Your Account & Registration", view! {
                    <div>
                        <p class="mb-3 text-xs">
                            "You must be at least 13 years old to use the App and have legal consent if you are a minor in your jurisdiction. You agree to:"
                        </p>
                        {bullet_list(account_bullets)}
                        <p class="mb-3 text-xs">
                            "We reserve the right to disable your account for violations of these Terms, laws, or for any activity harmful to our Services."
                        </p>
                    </div>
                })}

                {terms_section("2. Child Safety Standards (CSAE Policy)", view! {
                    <div>
                        <p class="mb-3 text-xs">
                            "Yral has zero tolerance for Child Sexual Abuse and Exploitation (CSAE). We:"
                        </p>
                        {bullet_list(csae_bullets)}
                    </div>
                })}

                {terms_section("3. Community Guidelines", view! {
                    <div>
                        <p class="mb-3 text-xs">
                            "Users are required to follow Yral's Community Guidelines and refrain from uploading or engaging in:"
                        </p>
                        {bullet_list(community_guidelines_bullets)}
                        <p class="mb-3 text-xs">
                            "Violation of these terms may result in content removal, account suspension, or reporting to authorities."
                        </p>
                    </div>
                })}

                {terms_section("4. Content Moderation & Reporting", view! {
                    <div>
                        <p class="mb-3 text-xs">
                            "Yral uses a mix of AI moderation tools and human reviewers to monitor content. Users can:"
                        </p>
                        {bullet_list(moderation_bullets)}
                        <p class="mb-3 text-xs">"We reserve the right to:"</p>
                        {bullet_list(moderation_rights_bullets)}
                    </div>
                })}

                {terms_section("5. User Responsibilities", view! {
                    <div>
                        <p class="mb-3 text-xs">"You agree to:"</p>
                        {bullet_list(user_responsibilities_bullets)}
                    </div>
                })}

                {terms_section("6. Blocking & Safety Controls", view! {
                    <div>
                        <p class="mb-3 text-xs">"Yral enables users to:"</p>
                        {bullet_list(blocking_bullets)}
                    </div>
                })}

                {terms_section("7. Content License", view! {
                    <div>
                        <p class="mb-3 text-xs">
                            "By posting content, you grant Yral a limited, non-exclusive, royalty-free license to use, display, and distribute your content solely for operating and promoting the platform. You retain ownership of your content."
                        </p>
                    </div>
                })}

                {terms_section("8. Account Termination", view! {
                    <div>
                        <p class="mb-3 text-xs">"We reserve the right to:"</p>
                        {bullet_list(vec![
                            "Suspend or delete your account at our sole discretion",
                            "Remove content that violates these Terms or any applicable law"
                        ])}
                        <p class="mb-3 text-xs">
                            "You may delete your account at any time via the app settings."
                        </p>
                    </div>
                })}

                {terms_section("9. Privacy Policy", view! {
                    <div>
                        <p class="mb-3 text-xs">
                            "Please refer to our "
                            <a href="/privacy-policy" class="underline text-blue-400">Privacy Policy</a>
                            " for full details on how we collect, use, and protect your data."
                        </p>
                    </div>
                })}

                {terms_section("10. Disclaimer and Limitation of Liability", view! {
                    <div>
                        <p class="mb-3 text-xs">
                            "The App is provided \"as is\" and without warranties. We are not liable for:"
                        </p>
                        {bullet_list(liability_bullets)}
                        <p class="mb-3 text-xs">"Use the App at your own risk."</p>
                    </div>
                })}

                {terms_section("11. Changes to Terms", view! {
                    <div>
                        <p class="mb-3 text-xs">
                            "We may update these Terms periodically. Continued use of the App after updates means you accept the new Terms."
                        </p>
                    </div>
                })}

                {terms_section("12. Contact Information", view! {
                    <div>
                        <p class="mb-3 text-xs">"For safety issues, CSAE reports, or support:"</p>
                        <p class="mb-3 text-xs">📧 Email: support@yral.com</p>
                    </div>
                })}
            </div>
        </div>
    }
} 