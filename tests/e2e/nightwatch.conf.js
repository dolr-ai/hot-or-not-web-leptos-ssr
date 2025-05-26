// Refer to the online docs for more details:
// https://nightwatchjs.org/gettingstarted/configuration/
//

//  _   _  _         _      _                     _          _
// | \ | |(_)       | |    | |                   | |        | |
// |  \| | _   __ _ | |__  | |_ __      __  __ _ | |_   ___ | |__
// | . ` || | / _` || '_ \ | __|\ \ /\ / / / _` || __| / __|| '_ \
// | |\  || || (_| || | | || |_  \ V  V / | (_| || |_ | (__ | | | |
// \_| \_/|_| \__, ||_| |_| \__|  \_/\_/   \__,_| \__| \___||_| |_|
//             __/ |
//            |___/

const percy = require('@percy/nightwatch');

module.exports = {
  // An array of folders (excluding subfolders) where your tests are located;
  // if this is not specified, the test source must be passed as the second argument to the test runner.
  src_folders: ['src'],

  // See https://nightwatchjs.org/guide/concepts/page-object-model.html
  // page_objects_path: ['nightwatch/page-objects'],

  // See https://nightwatchjs.org/guide/extending-nightwatch/adding-custom-commands.html
  custom_commands_path: [percy.path],

  // See https://nightwatchjs.org/guide/extending-nightwatch/adding-custom-assertions.html
  // custom_assertions_path: ['nightwatch/custom-assertions'],

  // See https://nightwatchjs.org/guide/extending-nightwatch/adding-plugins.html
  plugins: ['@percy/nightwatch'],

  // See https://nightwatchjs.org/guide/concepts/test-globals.html
  globals_path: '',

  globals: {
    asyncHookTimeout: 30000,
  },

  webdriver: {},
  test_workers: {
    enabled: true
  },

  test_settings: {
    default: {
      disable_output_boxes: true,
      disable_error_log: false,
      launch_url: '${PREVIEW_URL}',

      screenshots: {
        enabled: false,
        path: 'screens',
        on_failure: true
      },

      desiredCapabilities: {
        browserName: 'firefox'
      },

      webdriver: {
        start_process: true,
        server_path: ''
      },

    },

    chrome: {
      extends: 'default',
      desiredCapabilities: {
        browserName: 'chrome'
      }
    },

    browserstack: {
      selenium: {
        host: 'hub.browserstack.com',
        port: 443
      },
      // More info on configuring capabilities can be found on:
      // https://www.browserstack.com/automate/capabilities?tag=selenium-4
      desiredCapabilities: {
        'bstack:options': {
          userName: '${BROWSERSTACK_USER_NAME}',
          accessKey: '${BROWSERSTACK_ACCESS_KEY}',
          buildName: '${BUILD_NAME}',
          projectName: 'hot-or-not-web-leptos-ssr'
        }
      },

      disable_error_log: true,
      webdriver: {
        timeout_options: {
          timeout: 15000,
          retry_attempts: 3
        },
        keep_alive: true,
        start_process: false
      }
    },

    'browserstack.chrome': {
      extends: 'browserstack',
      desiredCapabilities: {
        browserName: 'chrome',
        chromeOptions: {
          w3c: true
        }
      }
    },

  },

};
