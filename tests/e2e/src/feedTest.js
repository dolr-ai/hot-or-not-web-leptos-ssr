// const grpc = require('@grpc/grpc-js');
// const protoLoader = require('@grpc/proto-loader');
// const path = require('path');

// describe('GRPC API Test', function() {
//   let client;

//   before(function(browser) {
//     // Load your proto file and create a client
//     const PROTO_PATH = path.resolve(__dirname, '../../../ssr/src/utils/contracts/projects/ml_feed/ml_feed.proto');
//     const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
//       keepCase: true,
//       longs: String,
//       enums: String,
//       defaults: true,
//       oneofs: true
//     });

//     const protoDescriptor = grpc.loadPackageDefinition(packageDefinition);
//     // Replace with your service name and endpoint
//     client = new protoDescriptor.ml_feed.MLFeed('yral-ml-feed-server.fly.dev:443', grpc.credentials.createSsl());
//   });

//   it('should successfully make a GRPC API call to get feed clean', function(browser) {
//     // Create a promise wrapper for the GRPC call
//     const makeGrpcCall = () => {
//       return new Promise((resolve, reject) => {
//         // Replace with your actual method and request data
//         client.get_feed_clean({ 
//             canister_id: "dyuzm-uqaaa-aaaal-agt7q-cai",
//             filter_posts: [],
//             num_results: 10
//          }, (error, response) => {
//           if (error) reject(error);
//           else resolve(response);
//         });
//       });
//     };

//     // Make the GRPC call and assert the response
//     browser.perform(async () => {
//       try {
//         const response = await makeGrpcCall();
//         browser.assert.ok(response, 'GRPC call succeeded');
//         // assert that the response is an array of length > 0
//         browser.assert.ok(response.feed.length > 0, 'Response feed array has items');
//       } catch (error) {
//         browser.assert.fail(`GRPC call failed: ${error.message}`);
//       }
//     });
//   });

//   it('should successfully make a GRPC API call to get feed nsfw', function(browser) {
//     // Create a promise wrapper for the GRPC call
//     const makeGrpcCall = () => {
//       return new Promise((resolve, reject) => {
//         // Replace with your actual method and request data
//         client.get_feed_nsfw({ 
//             canister_id: "dyuzm-uqaaa-aaaal-agt7q-cai",
//             filter_posts: [],
//             num_results: 10
//          }, (error, response) => {
//           if (error) reject(error);
//           else resolve(response);
//         });
//       });
//     };

//     // Make the GRPC call and assert the response
//     browser.perform(async () => {
//       try {
//         const response = await makeGrpcCall();
//         browser.assert.ok(response, 'GRPC call succeeded');
//         // assert that the response is an array of length > 0
//         browser.assert.ok(response.feed.length > 0, 'Response feed array has items');
//       } catch (error) {
//         browser.assert.fail(`GRPC call failed: ${error.message}`);
//       }
//     });
//   });

//   after(function(browser) {
//     browser.end();
//   });
// });
