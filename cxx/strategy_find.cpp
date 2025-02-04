#include <filesystem>
#include <iostream>

#include <boost/program_options.hpp>
namespace bpo = boost::program_options;

#include "game.hpp"

int main(int argc, char** argv) {
    unsigned max_clique = 2;
    unsigned max_colors = 2;
    unsigned max_intervals = 3;
    unsigned max_depth = 5;
    std::string start_state = "[]";

    std::ios_base::sync_with_stdio(false);
    std::string app_name = std::filesystem::path(argv[0]).stem().string();
    bpo::options_description desc("Options");
    desc.add_options()
        ("help", "Print help messages")
        ("clique", bpo::value<unsigned>(&max_clique), "Maximum clique size")
        ("colors", bpo::value<unsigned>(&max_colors), "Number of colors")
        ("intervals", bpo::value<unsigned>(&max_intervals), "Maximum number of active intervals")
        ("depth", bpo::value<unsigned>(&max_depth), "Maximum strategy depth")
        ("start", bpo::value<std::string>(&start_state), "Start state")
        ;
    bpo::positional_options_description pos;
    bpo::variables_map var_map;
    try {
        bpo::store(bpo::command_line_parser(argc, argv).options(desc).positional(pos).run(), var_map);
        if ( var_map.count("help") ) {
            std::cerr << app_name << std::endl;
            std::cerr << desc << std::endl;
            return 0;
        }
        bpo::notify(var_map);
    } catch ( bpo::error& e ) {
        std::cerr << "ERROR: " << e.what() << std::endl << std::endl;
        std::cerr << desc << std::endl;
        return 1;
    }

    State state;
    state.load(start_state);
    Game game(max_clique, max_colors, max_intervals, max_depth, state);
    //Game game(2, 2, 3, 5);
    //Game game(3, 4, 5, 7);
    //Game game(4, 6, 9, 13);
    //Game game(5, 8, 12, 22);
    std::cerr << game.solve() << std::endl;
    std::cerr << game << std::endl;
    return 0;
}
