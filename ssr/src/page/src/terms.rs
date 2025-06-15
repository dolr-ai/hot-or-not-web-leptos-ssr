use component::{back_btn::BackButton, title::TitleText};
use leptos::prelude::*;
use leptos_meta::*;
use state::app_state::AppState;

#[component]
pub fn TermsOfService() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Terms of Service";
    view! {
        <Title text=page_title />
        <div class="w-screen min-h-screen bg-black pt-4 pb-12 text-white flex flex-col items-center">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/menu".to_string() />
                    <span class="font-bold">Terms of Service</span>
                    <div></div>
                </div>
            </TitleText>
            <div class="px-8 flex h-full w-full flex-col space-y-8 overflow-hidden overflow-y-auto py-16">
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Introductions</div>
                    <div class="text-xs whitespace-pre-line">
                        {r#"Thanks for choosing Yral, an online social platform. This is an agreement between "you", a "user" of “Yral”, and GoBazzinga Inc., a company incorporated in the United States of America ("we", "us", "our''). The platform “Yral” has been developed by GoBazzinga Inc. and we operate all features of Yral and tokens in the application.
                        
                        "CENT token" refers to the blockchain-based unit known as 'CENT' built upon the Internet Computer protocol.
                        
                        Subject to the foregoing, we are pleased to make our services available to you, which include the Yral Web App, website, social and interactive features, which include image posting, video posting and other associated services ("Services"). These images, videos, clips, interactions etc. on Yral posted by the users shall be the "Content".
                        
                        If you install, use, access Yral or any of its features, you have read and accepted these terms and conditions. You have also accepted the Yral [privacy policy link], and other terms which are available on the Yral App, Yral Website, and otherwise. Please make sure to read these documents. By using Yral, you are entering into a binding contract(s) with us. Your contract with us includes the Terms and conditions and privacy policy etc. If you don’t agree with and accept the above, please do not use, install, access Yral or any of its features."""#}
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Account and Registration</div>
                    <div class="text-xs">
                        You must be over the age of 13, and you must be represented by a legal
                        guardian if you are below the age of 18, to register for our Services.
                        You represent and warrant that you will provide accurate and up to
                        date information while creating an account with us and you agree to
                        keep the information accurate at all times. You must keep your account
                        password confidential. You shall be solely responsible (to us and to
                        others) for the activity on your account and its consequences. If at
                        any time, if you fail to comply with the provision of these terms or
                        if activities occur on your account which might cause damage to the
                        Services or infringe or violate any third party rights or violate any
                        laws or regulations, or for any other reason, we reserve the right to,
                        at our discretion, disable your account and remove or disable any
                        Content you upload or share. Only persons who can form legally binding
                        contracts under the law of their jurisdiction or those persons (such
                        as minors) that are represented by persons (such as legal guardians)
                        who can form legally binding contracts under the law of their
                        jurisdiction, are permitted to use our Services. You agree that: <br />
                        <ul class="list-decimal py-2 pl-6">
                            <li>
                                You have the right, authority and legal capacity to agree to these
                                Terms and conditions, privacy policy and community guidelines on
                                behalf of yourself.
                            </li>
                            <li>
                                You have read, understood and agreed to be bound by these Terms
                                and conditions, privacy policy and community guidelines with
                                respect to yourself and any minor in your care who has access to
                                our Services.
                            </li>
                        </ul>
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Content posted by you</div>
                    <div class="text-xs">
                        {r#"By posting any Content on or through our Services, you hereby grant us
                        a non-exclusive, fully paid and royalty-free, worldwide, limited
                        licence to display such Content as felt appropriate, in any media
                        formats through any media channels, and delete such Content. We do not
                        claim any ownership or modification or derivative rights in any
                        Content or to the underlying works in the Content that you post on or
                        through our Services. You agree that your use of the Services does not
                        grant you any right to any compensation or share in revenue or value.
                        To the extent it's necessary, when you generate Content, you also
                        grant us, our affiliates, and our business partners the unrestricted,
                        worldwide, perpetual right and licence to use your name, likeness, and
                        voice. You will not be entitled to any compensation from us, our
                        affiliates, or our business partners if your name, likeness, or voice
                        is conveyed through the Services. You represent, warrant and covenant,
                        as applicable, that: (i) you own the Content posted by you on or
                        through our Services or otherwise have the right to grant the licence
                        set forth in this section, (ii) the posting and use of your Content on
                        or through our Services does not violate the privacy rights,
                        copyrights, contractual rights, intellectual property rights or any
                        other rights of any person, and (iii) the posting of your Content does
                        not result in a breach of contract between you and a third party (iv)
                        Your actions on Yral shall not be in violation of applicable law
                        or regulation. We are not responsible for any Content posted by you or
                        any consequences thereof. You agree that you shall not host, display,
                        upload, modify, publish, transmit, store, update or share any
                        information that:"""#} <ul class="list-decimal py-2 pl-6">
                            <li>
                                belongs to another person and to which you do not have any right
                                to;
                            </li>
                            <li>
                                {r#"is grossly harmful, harassing, blasphemous, defamatory, obscene,
                                pornographic, paedophilic, libellous, invasive of another's
                                privacy, including bodily privacy, insulting or harassing on the
                                basis of gender, libellous, hateful, racially or ethnically
                                objectionable, disparaging, relating or encouraging money
                                laundering or gambling, or otherwise unlawful inconsistent with or
                                contrary to the laws in force in any manner whatever;"""#}
                            </li>
                            <li>harms minors in any way;</li>
                            <li>
                                infringes any patent, trademark, copyright or other proprietary
                                rights;
                            </li>
                            <li>violates any law for the time being in force;</li>
                            <li>
                                deceives or misleads the addressee about the origin of such
                                messages or communicates any information which is grossly
                                offensive or menacing in nature or which is patently false or
                                misleading in nature but may reasonably be perceived as a fact;
                            </li>
                            <li>impersonates another person;</li>
                            <li>
                                contains software viruses or any other computer code, files or
                                programs designed to interrupt, destroy or limit the functionality
                                of any computer resource;
                            </li>
                            <li>
                                intimidates or harasses another, or promotes sexually explicit
                                material, violence or discrimination based on race, sex, religion,
                                nationality, disability, sexual orientation or age.
                            </li>
                            <li>
                                contains material that contains a threat of any kind, including
                                threats of physical violence;
                            </li>
                            <li>is slanderous or defamatory;</li>
                            <li>
                                is patently false and untrue, and is written or published in any
                                form, with the intent to mislead or harass a person, entity or
                                agency for financial gain or to cause any injury to any person.
                            </li>
                            {r#"You acknowledge and agree that the Content uploaded is the sole responsibility
                            of the user who has uploaded such Content and the views expressed are
                            their own. Yral is not responsible for the Content posted by you.
                            You shall be solely responsible for the Content posted, shared, modified,
                            uploaded, transmitted, updated and hosted by you. You agree that your
                            use of Yral's services will conform to the Community Guidelines.
                            You agree that in case of non-compliance with applicable laws, or with
                            these terms and conditions, or with the Yral privacy policy or
                            with our community Guidelines, we have the right to terminate your access
                            or usage rights to Yral immediately and remove non-compliant content
                            from Yral. You hereby agree that you will never use Yral
                            in violation of any applicable law. If any violation of these Terms is
                            brought to our actual knowledge by an affected person, we shall act within
                            thirty-six hours and where applicable, work with the user or owner of
                            such information to delete/disable such information/Content that is in
                            contravention of these Terms. We will not intimate you if any of the
                            Content posted by you is taken down / deleted or your account is disabled
                            pursuant to the above. We will preserve such information and associated
                            records for at least one hundred and eighty days for investigation purposes.
                            Please contact the grievance officer whose details are provided in the
                            last section of these Terms, in case of any violation or grievances."""#}
                        </ul>
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">NSFW Content Policy</div>
                    <div class="text-xs">
                        By enabling the NSFW (Not Safe for Work) content toggle, you acknowledge and consent to viewing adult-oriented
                        content within your feed. This content may include themes that are intended for mature audiences.
                        You agree that: <ul class="list-decimal py-2 pl-6">
                            <li>You are of legal age to view such content in your jurisdiction.</li>
                            <li>
                                The platform is not responsible for any distress or offense caused by NSFW content.
                            </li>
                            <li>
                                You may disable NSFW content at any time by turning off the toggle in your settings.
                            </li>
                        </ul>
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">{r#"Children's Privacy"#}</div>
                    <div class="text-xs">
                        These Services are not for any child under the age of 13. We do not
                        knowingly collect personally identifiable information from children
                        under 13. In case we discover that a child under 13 has provided us
                        with personal information, we will delete this information from our
                        servers or restrict the access to such information by the user and by
                        others. If you are a parent or guardian and you are aware that your
                        child has provided us with personal information, please contact the
                        grievance officer whose details are provided in the last section of
                        these Terms.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Devices and Software</div>
                    <div class="text-xs">
                        You must have certain devices, software, and data connections to use
                        our Services, which we otherwise do not supply. For as long as you use
                        our Services, you consent to downloading and installing updates to our
                        Services, including automatically.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Device Storage Permission</div>
                    <div class="text-xs">
                        You provide us storage permission (Read and write) by which we are
                        able to upload your content to our server and display it to users. You
                        confirm you are authorised to provide us such permissions to allow us
                        to offer our Services.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Privacy policy and user data</div>
                    <div class="text-xs">
                        yral.com cares about your privacy. You can access our privacy policy
                        from the menu.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Changes to these Terms</div>
                    <div class="text-xs">
                        {r#"We may amend or update these Terms. We may update the “Last Modified” date at the top of these Terms. Your continued use of Yral confirms your acceptance of these Terms, as amended. If you do not agree with or accept these Terms, as amended, you must stop using Yral. Please review these Terms from time to time. These changes are effective immediately after they are posted on this page."""#}
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Indemnity</div>
                    <div class="text-xs">
                        {r#"You agree to defend, indemnify, and hold harmless us, our parent(s),
                        subsidiaries, and affiliates, and each of their respective officers,
                        directors, employees, agents and advisors from any and all claims,
                        liabilities, costs, and expenses, including, but not limited to,
                        attorneys' fees and expenses, arising out of a breach by you or any
                        user of your account of these Terms or the privacy policy or community
                        guidelines and your obligations, representation and warranties
                        therein."""#}
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Exclusion of Warranties</div>
                    <div class="text-xs">
                        {r#"The Services are provided to you on an 'as is' basis and we do not
                        represent or warrant that the use of the Services will be
                        uninterrupted, timely, secure or free from error; that any information
                        obtained by you as a result of the use of services will be accurate.
                        Implied warranties as to the satisfactory quality, fitness for purpose
                        or merchantability of these Services are hereby excluded."""#}
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Limitation of liability</div>
                    <div class="text-xs">
                        We shall not be liable to you for any loss or damages or claims
                        including but not limited to the following:
                        <ul class="list-decimal py-2 pl-6">
                            <li>Any loss of profit, opportunity or goodwill;</li>
                            <li>Any loss of data;</li>
                            <li>
                                Any damage incurred as a result of your reliance on any
                                advertisement appearing on the Service;
                            </li>
                            <li>
                                Any damage incurred as a result of any changes to the Service, its
                                features and any temporary or permanent termination of our
                                Services;
                            </li>
                            <li>
                                Any damage incurred as a result of your failure to provide us with
                                accurate information and your failure to keep your account details
                                and password confidential and secure;
                            </li>
                            <li>Any damage incurred as a result of the actions of another user.</li>
                        </ul>Any dispute that may arise between you and any third party arising from
                        your use of the Services shall be only between you and a third party and
                        you release us and our affiliates from any such claims and damages connected
                        with such disputes.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Governing Law</div>
                    <div class="text-xs">
                        The validity, construction and enforceability of these Terms and
                        conditions , privacy policy and community guidelines shall be governed
                        and construed in accordance with the laws of the state of Delaware in
                        the the United States of America.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Severability</div>
                    <div class="text-xs">
                        If any provision of this Agreement is invalid or unenforceable or
                        prohibited by law, it shall be treated for all purposes as severed
                        from this Agreement and ineffective to the extent of such invalidity
                        or unenforceability, without affecting in any way the remaining
                        provisions hereof, which shall continue to be valid and binding.
                    </div>
                </div>
                <div class="flex flex-col space-y-2">
                    <div class="text-sm font-semibold">Grievance Redressal</div>
                    <div class="text-xs">
                        If you wish to complain about our Services / the use of our services
                        by others, please contact our Resident Grievance Officer, whose
                        details are below: <div class="pl-4 pt-2">Name: Utkarsh Goyal</div>
                        <div class="pl-4">Designation: Director</div>
                        <div class="pb-2 pl-4">Email ID: support@gobazzinga.io</div>
                        We will endeavour to redress the complaint within three months from the
                        date of receipt of the complaint.
                    </div>
                </div>
            </div>
        </div>
    }
}
