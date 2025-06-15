use leptos::prelude::*;
use leptos_meta::*;

use component::{back_btn::BackButton, title::TitleText};
use state::app_state::AppState;

#[component]
pub fn PrivacyPolicy() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Privacy Policy";
    view! {
        <Title text=page_title />
        <div class="flex flex-col items-center pt-4 pb-12 w-screen min-h-screen text-white bg-black">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/menu".to_string() />
                    <div>
                        <span class="font-bold">Privacy Policy</span>
                    </div>
                    <div></div>
                </div>
            </TitleText>
            <div class="flex overflow-hidden overflow-y-auto flex-col py-16 px-8 space-y-8 w-full h-full">
                <div class="text-xs whitespace-pre-line">
                    {r#"Thank you for choosing "Yral." We are delighted to provide our Service to you. "Yral" encompasses the Yral App, website, features, and associated services. It offers image and video posting capabilities along with various social and interactive features. This Privacy Policy outlines our information practices.
                    
                    By installing, using, or accessing "Yral" or any of its features, you acknowledge that you have read and accepted the terms of this Policy. Additionally, you agree to the "Yral" Terms of Service, and other applicable terms available on the "Yral" App and Website. If you do not agree with these terms, please refrain from using "Yral" or any of its features.
                    
                    This Policy applies to information provided by users while using "Yral" and not to information obtained by "Yral" from other sources. Information acquired from third-party websites, apps, landing pages, etc., accessed through "Yral" by users, will be subject to the privacy policies and terms of those respective platforms.
                    
                    You also warrant and represent that any registration information and other information that you submit or otherwise provide to Yral is true, accurate and complete, and you agree to keep it that way at all times.
                    "#}
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Information We Collect</div>
                    <div class="text-xs">
                        The below information is collected by Yral: <ul class="pl-6 list-decimal">
                            <li>
                                Information collected with a prompt: <ul class="pl-4 list-disc">
                                    <li>
                                        In order to use Yral, we may need you to authenticate your account using your google account.
                                    </li>
                                    <li>
                                        {r#"We may ask you for the location information and regional preferences to customise your experience by showing you relevant content. Yral may collect and store the current and past location of your device if you permit. You have full control over whether to allow Yral to collect this information by making changes to your device’s settings."#}
                                    </li>
                                    <li>
                                        Customer Support. You may also provide us with information related to your use of Yral, including copies of your messages / posts, and how to contact you so that we can provide you with customer support. For example, you may send us an email with information relating to our app performance, or other issues.
                                    </li>
                                </ul>
                            </li>
                            <li>
                                Information collected without a prompt: <ul class="pl-4 list-disc">
                                    <li>
                                        Device Information: We may collect the below with respect to
                                        any device through which Yral is used or accessed:
                                        <ul class="pl-4 list-disc">
                                            <li>Device Identifiers.</li>
                                            <li>IP Address.</li>
                                            <li>Operating System.</li>
                                            <li>Operating System Version.</li>
                                            <li>Advertising Identifiers.</li>
                                            <li>Device Make.</li>
                                            <li>Display Features.</li>
                                            <li>Network Provider/Wi-Fi or other ISP.</li>
                                            <li>Battery level and usage.</li>
                                            <li>Memory usage.</li>
                                            <li>
                                                Other apps or services installed or used through the
                                                device.
                                            </li>
                                            <li>
                                                Content Type and Non-Personal information related to ads.
                                            </li>
                                            <li>Profile Information.</li>
                                        </ul>
                                    </li>
                                    <li>
                                        Information related to your use of Yral such as content
                                        preferences and consumption, interaction with other users,
                                        user activity, etc.
                                    </li>
                                    <li>
                                        {r#"Yral may access your device’s camera, microphone, photo
                                        and/or video and/or audio library or gallery."#}
                                    </li>
                                    <li>
                                        Yral may also collect information provided by third
                                        parties who may disclose to Yral personal or
                                        non-personal information collected by them. Yral has no
                                        control over or responsibility in respect of third-party
                                        information practices or information collection methods.
                                    </li>
                                    <li>
                                        Yral may also collect tracking information including
                                        cookies, DART, beacons, etc.
                                    </li>
                                    <li>
                                        Usage and Log Information. We collect service-related,
                                        diagnostic, and performance information. This includes
                                        information about your activity (such as how you use Yral,
                                        how you interact with others using Yral, etc.), log
                                        files, and diagnostics, crash information, website, and
                                        performance logs and reports, etc.
                                    </li>
                                </ul>
                            </li>
                        </ul>
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm">Information We Do Not Collect</div>
                    <div class="text-xs">
                        We do not collect any Sensitive Personal Data or information such as
                        password related to other services, financial information such as bank
                        account or credit card or debit card or other payment instrument
                        details, physical, physiological and mental health condition, sexual
                        orientation, medical records and history, or biometric information.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm">How We Use Information</div>
                    <div class="text-xs">
                        We may use all the information we have to help us operate, provide,
                        improve, understand, customise, support, and market Yral.
                        <ul class="pl-6 list-decimal">
                            <li>
                                Our Services: We operate and provide Yral, and improving,
                                fixing, and customising Yral by using the said information.
                                We understand how users use Yral, and analyse and use the
                                information that we have to evaluate and improve Yral,
                                research, develop, and test new services and features, and conduct
                                troubleshooting activities. We also use your information to
                                respond to you when you contact us. We may also use your
                                information for (i) displaying content based on interest,
                                location, offers, etc., (ii) displaying or providing access to
                                promotions, advertisements, offer, etc. which may be based on your
                                interests and also ad-targeting, ad-placement, and ad-measurement
                                (iii) displaying location specific news and weather related
                                information, (iv) improving search results, content loading,
                                performance, etc.
                            </li>
                            <li>
                                Safety and Security: We also use your information to try and
                                ensure safety and security of our users and their information. We
                                verify accounts and activity, and promote safety and security on
                                and off Yral, such as by investigating suspicious activity
                                or violations of our policies, terms, etc., and to try and ensure
                                that Yral is being used legally.
                            </li>
                        </ul>
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">How We Share Information</div>
                    <div class="text-xs">
                        We may share your information in ways including the following:
                        <ul class="pl-6 list-decimal">
                            <li>
                                With other Yral users: your username, profile photograph,
                                content that you post or send or share. You may also be able to
                                control how your content is shared with other Yral users by
                                adjusting personal settings.
                            </li>
                            <li>
                                With our business partners, affiliates, investors: public
                                information like your name, username and profile pictures and any
                                content posted by you.
                            </li>
                            <li>
                                We may share information about you with third party service
                                providers who perform services on our behalf, including to measure
                                and optimise the performance of ads and deliver more relevant ads,
                                including on third-party websites and apps.
                            </li>
                            <li>
                                We may share information about you, such as device and usage
                                information, to help us and others prevent fraud.
                            </li>
                            <li>
                                We may share information about you for legal, safety, and security
                                reasons.
                            </li>
                            <li>
                                We may share information about you if we reasonably believe that
                                disclosing the information is needed to: (i) comply with any valid
                                legal process, governmental request, or applicable law, rule, or
                                regulation, (ii) investigate, remedy, or enforce potential
                                violations of this Policy, the Terms of Service and the Community
                                Guidelines.
                            </li>
                            <li>
                                To protect the rights, property, or safety of us, our users, or
                                others.
                            </li>
                            <li>To detect and resolve any fraud or security concerns.</li>
                        </ul>
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">To manage access to your Google account</div>
                    <div class="text-xs">
                        <ul class="pl-6 list-decimal">
                            <li>Visit your Google Account settings.</li>
                            <li>Navigate to the "Security"section.</li>
                            <li>
                                Look for the "Third-party apps with account access"
                                or similar option. Here, you can view and manage the apps connected to your Google account.
                            </li>
                            <li>Locate "Yral"inthelistofconnected apps.</li>
                            <li>
                                You can adjust permissions or revoke access to your Google account as needed.
                            </li>
                        </ul>
                    </div>
                    <div class="text-xs">
                        {r#"Ensure to review and manage app permissions periodically to maintain control over your account's access rights and security.
                        You can read about how Google helps users share their data safely here: https://safety.google/privacy/privacy-controls/"#}
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Security</div>
                    <div class="text-xs">
                        We undertake no responsibility for the security or safety of your
                        information. However, we take reasonable security measures to protect
                        your information in accordance with industry standards. This does not
                        guarantee safety or security of your information as events beyond our
                        control can also occur.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">
                        Assignment, Change Of Control, And Transfer
                    </div>
                    <div class="text-xs">
                        All of our rights and obligations under this Policy, the Terms of
                        Service and the Community Guidelines are freely assignable by us to
                        any of our affiliates or any others, in connection with a merger,
                        acquisition, restructuring, or sale of assets, or by operation of law
                        or otherwise, and we may transfer your information to any of our
                        affiliates, any others successor entities, or new management, etc.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Law And Protection</div>
                    <div class="text-xs">
                        We may collect, use, preserve, and share your information if we
                        believe that it may be reasonably necessary to: (a) respond pursuant
                        to applicable law or regulations, to legal process, or to government
                        requests; (i) comply with any valid legal process, governmental
                        request, or applicable law, rule, or regulation, (ii) investigate,
                        remedy, or enforce potential violations of this Policy, the Terms of
                        Service and Community Guidelines, (iii) to protect the rights,
                        property, or safety of us, our users, or others, (iv) to detect and
                        resolve any fraud or security concerns or technical issues, etc., (iv)
                        for other legal requirements, etc.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Updates To Our Policy</div>
                    <div class="text-xs">
                        {r#"We may amend or update this Policy. We will provide you notice of amendments to this Policy, as appropriate, and update the “Last Modified” date at the top of this Policy. Your continued use of Yral confirms your acceptance of this Policy, as amended. If you do not agree with or accept this Policy, as amended, you must stop using Yral. Please review our Privacy Policy from time to time."#}
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Contact Us</div>
                    <div class="text-xs">
                        {r#"If you have questions about this Policy, email at : support@gobazzinga.io"#}
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Governing Law and Jurisdiction</div>
                    <div class="text-xs">
                        The validity, construction and enforceability of these Terms and
                        Content Policy, the Privacy Policy shall be governed and construed in
                        accordance with the laws of the state of Delaware, United States
                        of America.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Withdrawal of consent</div>
                    <div class="text-xs">
                        You also have the right to withdraw your consent at any time and
                        request us to stop processing your data. However, withdrawing your
                        consent will not affect the processing or use of your information
                        prior to fulfilling your consent withdrawal request. You can email us
                        with your registered email address.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Right to opt out</div>
                    <div class="text-xs">
                        {r#"You can opt out from our use of data from cookies and similar
                        technologies that track your behaviour on the sites of others for ad
                        targeting and other ad-related purposes. You can also opt out of
                        marketing communication. We will ask you to opt-in before we use GPS
                        or other tools to identify your precise location. However, you will
                        continue to receive important emails relating to your account. You can
                        email us with your registered email address to unsubscribe from such
                        communications."#}
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Third party Links</div>
                    <div class="text-xs">
                        We are not responsible for any content nor any consequences either
                        directly or indirectly arising out of clicking on any third party link
                        you may come across posted by other users. We may share your
                        information with approved third parties in order to provide our
                        services through the Yral App
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Data Transfers</div>
                    <div class="text-xs">
                        We may transfer personal information to countries other than the
                        country in which the data was originally collected.
                    </div>
                </div>
            </div>
        </div>
    }
}
