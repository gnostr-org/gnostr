// This file is part of 'Nostr_client_relay'
// Copyright (c) 2023, Space Research Software LLC, Pedro Vicente. All rights reserved.
// See file LICENSE for full license details.

#include <string>
#include <vector>
#include <fstream>
#include <iostream>
#include <sstream>
#include <future>
#include <algorithm>


#include "../include/pstream.h"
#include "../src/uuid.hh"
#include "../src/log.hh"
#include "../src/nostr.hh"
#include "../src/argparse.hpp"
#include "../include/sha256.h"

std::string log_program_name("tests");

/////////////////////////////////////////////////////////////////////////////////////////////////////
// prototypes
/////////////////////////////////////////////////////////////////////////////////////////////////////

int read_list(const std::string& file_name);
int read_lists();
int relay_all(const std::string& json);
int get_feed();
int get_metadata();
int gnostr();
int gnostr_get_relays();
int gnostr_sha256();

std::vector<std::string> relays = { "eden.nostr.land",
  "nos.lol",
  "relay.snort.social",
  "relay.damus.io",
  "nostr.wine",
  "0.0.0.0:6102"
};

std::vector<std::string> list = { "list_01.txt",
"list_02.txt",
};

/////////////////////////////////////////////////////////////////////////////////////////////////////
// main
/////////////////////////////////////////////////////////////////////////////////////////////////////

int main(int argc, char * argv[])
{

    comm::start_log();

    std::string uuid = uuid::generate_uuid_v4();
    //comm::log(uuid);
    //std::cout << uuid << std::endl;

    argparse::ArgumentParser program("gnostr-tests", "v0.0.0");

    //positional arg
    program.add_argument("port").default_value<int>(0)
        .help("gnostr-tests <int>")
        .scan<'i', int>();

    program.add_argument("--port").default_value<int>(0)
        .nargs(1)
        .help("gnostr-tests --port <int>")
        .scan<'i', int>();

    try {
        program.parse_args(argc, argv);
    } catch (const std::runtime_error &err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        return 1;
    }

    //handle positional arg `port`
    int port = program.get<int>("port");
    if (port){
    std::cout << std::string("port:") << port << std::endl;
    }
    int port2 = program.get<int>("--port");
    if (port2){
        std::cout << std::string("--port:") << port2 << std::endl;
        std::cout << std::string("port2") << std::endl;
    }

    gnostr();
    gnostr_get_relays();
    gnostr_sha256();
    get_metadata();

    return 0;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
// read_lists
/////////////////////////////////////////////////////////////////////////////////////////////////////

int read_lists()
{
  for (int idx = 0; idx < list.size(); idx++)
  {
    if (read_list(list.at(idx)) < 0)
    {
      return -1;
    }
  }
  return 0;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
// read_list
/////////////////////////////////////////////////////////////////////////////////////////////////////

int read_list(const std::string& file_name)
{
  std::ifstream ifs;
  ifs.open(file_name);
  if (!ifs.is_open())
  {
    return -1;
  }
  std::string json;
  while (std::getline(ifs, json))
  {
    std::cout << json << std::endl;
    nostr::Type Type = nostr::get_message_type(json);

    /////////////////////////////////////////////////////////////////////////////////////////////////////
    // EVENT
    /////////////////////////////////////////////////////////////////////////////////////////////////////

    if (nostr::Type::EVENT == Type)
    {
      std::string event_id;
      nostr::event_t ev;
      if (nostr::parse_relay_event(json, event_id, ev) < 0)
      {
        assert(0);
      }
      comm::json_to_file("event.json", json);
      std::cout << event_id << std::endl;
      std::cout << ev.content << std::endl;
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////////
    // REQ
    /////////////////////////////////////////////////////////////////////////////////////////////////////

    else if (nostr::Type::REQ == Type)
    {
      std::string request_id;
      nostr::filter_t filter;
      if (nostr::parse_request(json, request_id, filter) < 0)
      {
        assert(0);
      }
      comm::json_to_file("request.json", json);
      std::cout << request_id << std::endl;
      if (filter.authors.size())
      {
        for (int idx = 0; idx < filter.authors.size(); idx++)
        {
          std::cout << filter.authors.at(idx) << " ";
        }
        std::cout << std::endl;
      }
    }
  }
  ifs.close();
  return 0;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
// relay_all
/////////////////////////////////////////////////////////////////////////////////////////////////////

int relay_all(const std::string& json)
{
  for (int idx = 0; idx < relays.size(); idx++)
  {
    std::string uri = relays.at(idx);
    std::vector<std::string> store;
    if (nostr::relay_to(uri, json, store) < 0)
    {
    }
  }
  return 0;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
// get_feed
/////////////////////////////////////////////////////////////////////////////////////////////////////

int get_feed()
{
  std::vector<std::string> pubkeys;
  std::string uri = "nos.lol";
  //const std::string pubkey("4ea843d54a8fdab39aa45f61f19f3ff79cc19385370f6a272dda81fade0a052b");
  //gnostr --sec $(gnostr-sha256)
  const std::string pubkey("a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd");

  /////////////////////////////////////////////////////////////////////////////////////////////////////
  // get_follows returns a list of pubkeys
  /////////////////////////////////////////////////////////////////////////////////////////////////////

  nostr::get_follows(uri, pubkey, pubkeys);
  comm::to_file("pubkeys.txt", pubkeys);

  /////////////////////////////////////////////////////////////////////////////////////////////////////
  // get feed returns an array of JSON events
  /////////////////////////////////////////////////////////////////////////////////////////////////////

  for (int idx_key = 0; idx_key < pubkeys.size(); idx_key++)
  {
    std::string pubkey = pubkeys.at(idx_key);

    std::vector<std::string> events;
    if (nostr::get_feed(uri, pubkey, events) < 0)
    {
    }
  }

  return 0;
}


/////////////////////////////////////////////////////////////////////////////////////////////////////
// get_metadata
/////////////////////////////////////////////////////////////////////////////////////////////////////

int get_metadata()
{
  std::vector<std::string> pubkeys;
  std::string uri = "nos.lol";
  //const std::string pubkey("4ea843d54a8fdab39aa45f61f19f3ff79cc19385370f6a272dda81fade0a052b");
  //gnostr --sec $(gnostr-sha256)
  const std::string pubkey("a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd");
  /////////////////////////////////////////////////////////////////////////////////////////////////////
  // get_follows returns a list of pubkeys
  /////////////////////////////////////////////////////////////////////////////////////////////////////

  nostr::get_follows(uri, pubkey, pubkeys);
  comm::to_file("pubkeys.txt", pubkeys);

  /////////////////////////////////////////////////////////////////////////////////////////////////////
  // metadata
  /////////////////////////////////////////////////////////////////////////////////////////////////////

  for (int idx_key = 0; idx_key < pubkeys.size(); idx_key++)
  {
    std::string pubkey = pubkeys.at(idx_key);

    std::vector<std::string> metadata;
    if (nostr::get_metadata(uri, pubkey, metadata) < 0)
    {
    }

    std::string message = metadata.at(0);
    try
    {
      nlohmann::json js = nlohmann::json::parse(message);
      std::string type = js.at(0);
      if (type.compare("EVENT") == 0)
      {
        nostr::event_t ev;
        from_json(js.at(2), ev);
        std::stringstream s;
        s << "metadata_content." << idx_key + 1 << ".json";
        comm::json_to_file(s.str(), ev.content);
      }
    }
    catch (const std::exception& e)
    {
      comm::log(e.what());
    }
  }

  return 0;
}

int gnostr(){

    redi::ipstream proc("./gnostr", redi::pstreams::pstdout | redi::pstreams::pstderr);
    std::string line;
    // read child's stdout
    while (std::getline(proc.out(), line))
      std::cout << "stdout: " << line << '\n';
    // if reading stdout stopped at EOF then reset the state:
    if (proc.eof() && proc.fail())
      proc.clear();
    // read child's stderr
    while (std::getline(proc.err(), line))
      std::cout << "stderr: " << line << '\n';

  return 0;
}

int gnostr_get_relays(){

    std::string gcc_command = "gcc ";
    gcc_command = gcc_command + " -o gnostr-get-relays " + "src/gnostr-get-relays.c";
    const char* compile = gcc_command.c_str();
    system(compile);

    redi::ipstream proc2("./gnostr-get-relays", redi::pstreams::pstdout | redi::pstreams::pstderr);
    std::string line;
    // read child's stdout
    while (std::getline(proc2.out(), line))
      std::cout << "stdout: " << line << '\n';
    // if reading stdout stopped at EOF then reset the state:
    if (proc2.eof() && proc2.fail())
      proc2.clear();
    // read child's stderr
    while (std::getline(proc2.err(), line))
      std::cout << "stderr: " << line << '\n';

  return 0;
}

int gnostr_sha256(){

    int gnostr_sha256 = system("gnostr-sha256");
    //std::cout << gnostr_sha256 << std::endl;
    int gnostr_weeble = system("gnostr-weeble");
    int gnostr_blockheight = system("gnostr-blockheight");
    int gnostr_wobble = system("gnostr-wobble");

    int returnCode = system("");
    // checking if the command was executed successfully
    if (returnCode == 0) {
      // std::cout << "Command executed successfully." << std::endl;
    }
    else {
      std::cout << "Command execution failed or returned "
                "non-zero: "
             << returnCode << std::endl;
    }
    // checking if the command was executed successfully
    // if (returnCode == 0) {
    //   std::cout << "Command executed successfully." << std::endl;
    // }
    // else {
    //   std::cout << "Command execution failed or returned "
    //             "non-zero: "
    //          << returnCode << std::endl;
    // }

    return 0;

}

