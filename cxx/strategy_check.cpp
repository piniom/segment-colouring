#include <cassert>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stack>
#include <unordered_map>

#include <boost/program_options.hpp>
namespace bpo = boost::program_options;

#include "state.hpp"

std::unordered_map<StateCode, size_t> known_codes;
std::vector<StateCode> codes;
std::vector<Interval> moves;
std::vector<std::vector<size_t>> responses;

int main(int argc, char** argv) {
    bool debug(false);
    std::vector<std::string> input_files(1, "-");

    std::ios_base::sync_with_stdio(false);
    std::string app_name = std::filesystem::path(argv[0]).stem().string();
    bpo::options_description desc("Options");
    desc.add_options()
        ("help", "Print help messages")
        ("debug", "Print debug information")
        ("input-file", bpo::value<std::vector<std::string>>(&input_files), "Input file")
        ;
    bpo::positional_options_description pos;
    pos.add("input-file", -1);
    bpo::variables_map var_map;
    try {
        bpo::store(bpo::command_line_parser(argc, argv).options(desc).positional(pos).run(), var_map);
        bpo::notify(var_map);
        if ( var_map.count("help") ) {
            std::cerr << app_name << std::endl;
            std::cerr << desc << std::endl;
            return 0;
        }
        if ( var_map.count("debug") )
            debug = true;
    } catch ( bpo::error& e ) {
        std::cerr << "ERROR: " << e.what() << std::endl << std::endl;
        std::cerr << desc << std::endl;
        return 1;
    }

    size_t max_clique = 0;
    size_t max_colors = 0;
    size_t max_size = 0;
    size_t max_depth = 0;
    for ( const auto& src : input_files ) {
        std::istream* stream = &std::cin;
        if ( src != "-" ) {
            stream = new std::ifstream(src.c_str());
            if ( not stream->good() ) {
                std::cerr << "Failed to load file '" << src << "'." << std::endl;
                return 1;
            }
        }
        size_t new_max_clique;
        size_t new_max_colors;
        for ( std::string line ; std::getline(*stream, line) ; ) {
            auto first = line.find_first_not_of(" \t\r\n");
            if ( first != std::string::npos and line[first] != '#' and line[first] != ';' ) {
                std::istringstream linestream(line);
                linestream >> new_max_clique >> new_max_colors;
                new_max_colors--;
                break;
            }
        }
        assert( new_max_clique >= 1 and new_max_clique <= 10 );
        assert( new_max_colors >= 1 and new_max_colors <= 20 );
        if ( (max_clique != 0 and new_max_clique != max_clique) or (max_colors != 0 and new_max_colors != max_colors ) ) {
            std::cerr << "Different input files play in different games." << std::endl;
            return 1;
        }
        max_clique = new_max_clique;
        max_colors = new_max_colors;
        std::cout << "Maximum clique size is " << max_clique << "." << std::endl;
        std::cout << "Maximum color is " << max_colors << "." << std::endl;
        for ( std::string line ; std::getline(*stream, line) ; ) {
            auto first = line.find_first_not_of(" \t\r\n");
            if ( first != std::string::npos and line[first] != '#' and line[first] != ';' ) {
                std::istringstream linestream(line);
                std::string state_code;
                linestream >> state_code;
                State state;
                state.load(state_code);
                if ( state.dump() != state_code ) {
                    std::cerr << "Strange state " << state_code << " (!=" << state.dump() << ")." << std::endl;
                    return 1;
                }
                auto code = state.encode();
                State rev = state;
                rev.reverse();
                auto rev_code = rev.encode();
                max_size = std::max<size_t>(max_size, state.get_interval_count());
                Interval interval;
                std::string state_move;
                linestream >> state_move;
                if ( state_move == "<" ) {
                    // PUSH RIGHT BARRIER
                    interval = Interval(0, 255);
                } else if ( state_move == ">" ) {
                    // PUSH LEFT BARRIER
                    interval = Interval(1, 255);
                } else {
                    // INTRODUCE INTERVAL
                    size_t move_left = std::stoi(state_move);
                    linestream >> state_move;
                    size_t move_right = std::stoi(state_move);
                    interval = Interval(move_left, move_right);
                }
                if ( known_codes.find(code) != known_codes.end() ) {
                    if ( moves[known_codes[code]] == interval )
                        continue;
                    std::cerr << "Multiple definition for " << state_code << "." << std::endl;
                    return 1;
                }
                known_codes[code] = codes.size();
                known_codes[rev_code] = codes.size();
                codes.emplace_back(code);
                moves.emplace_back(interval);
            }
        }
        if ( src != "-" ) {
            delete stream;
        }
    }
    std::cout << "Strategy has " << codes.size() << " states." << std::endl;
    std::cout << "Maximum state size is " << max_size << "." << std::endl;
    responses.resize(codes.size());
    for ( size_t i = 0 ; i < codes.size() ; i++ ) {
        State state;
        state.decode(codes[i]);
        auto move = moves[i];
        if ( move == Interval(0, 255) or move == Interval(1, 255) ) {
            if ( state.get_left_barrier() >= state.get_right_barrier() ) {
                std::cerr << "Cannot push barrier in " << state.dump() << "." << std::endl;
                return 1;
            }
            State result = state;
            if ( move == Interval(0, 255) )
                result.push_right_barrier();
            else
                result.push_left_barrier();
            auto code = result.encode();
            if ( known_codes.find(code) == known_codes.end() ) {
                std::cerr << "Answer for " << result.dump() << " not found in the strategy." << std::endl;
                return 1;
            }
            responses[i].emplace_back(known_codes[code]);
        } else {
            bool found = false;
            for ( const auto& move_responses : state.get_possible_additions(max_clique, max_colors) ) {
                if ( move_responses.first == move ) {
                    found = true;
                    for ( auto& response : move_responses.second ) {
                        ColoredInterval cint(move, response);
                        State result = state;
                        result.add_interval(cint);
                        auto code = result.encode();
                        if ( known_codes.find(code) == known_codes.end() ) {
                            std::cerr << "Answer for " << result.dump() << " not found in the strategy." << std::endl;
                            return 1;
                        }
                        responses[i].emplace_back(known_codes[code]);
                    }
                }
            }
            if ( not found ) {
                std::cerr << "Move for " << state.dump() << " is not valid." << std::endl;
                return 1;
            }

        }
        if ( debug ) {
            std::cout << " State #" << i << " is " << state.dump() << " with move ";
            if ( move == Interval(0, 255) )
                std::cout << "push one right";
            else if ( move == Interval(1, 255) )
                std::cout << "push one left";
            else
                std::cout << move;
            std::cout << " and ";
            if ( responses[i].size() == 0 )
                std::cout << "no possible responses";
            else {
                std::cout << "possible responses {";
                for ( const auto& response : responses[i] ) {
                    std::cout << " " << response;
                }
                std::cout << " }";
            }
            std::cout << std::endl;
        }
    }
    std::vector<std::vector<size_t>> inverse(codes.size());
    std::vector<size_t> in_counter(codes.size(), 0);
    std::vector<size_t> depth(codes.size(), 0);
    std::stack<size_t> wins;
    size_t win_count = 0;
    for ( size_t i = 0 ; i < codes.size() ; i++ ) {
        if ( responses[i].size() == 0 ) {
            win_count++;
            wins.push(i);
            depth[i] = 1;
        }
        for ( auto response : responses[i] ) {
            inverse[response].emplace_back(i);
        }
    }
    while ( wins.size() > 0 ) {
        size_t win = wins.top(); wins.pop();
        for ( size_t other : inverse[win] ) {
            in_counter[other]++;
            depth[other] = std::max(depth[other], depth[win]+1);
            if ( in_counter[other] == responses[other].size() ) {
                win_count++;
                wins.push(other);
            }
        }
    }
    if ( win_count == codes.size() ) {
        std::cout << "Strategy is correct." << std::endl;
        for ( size_t i = 0 ; i < codes.size() ; i++ ) {
            max_depth = std::max(max_depth, depth[i]);
        }
        std::cout << "Maximum strategy depth is " << max_depth << "." << std::endl;
    } else {
        std::cout << "Strategy is not correct. " << win_count << "/" << codes.size() << " states are wins." << std::endl;
        return 1;
    }
    State state;
    if ( known_codes.find(state.encode()) == known_codes.end() )
        std::cout << "Base state is not included in the strategy." << std::endl;
    return 0;
}
