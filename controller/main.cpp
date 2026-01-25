#include <readline/readline.h>
#include <readline/history.h>
#include <iostream>
#include <string>

int main() {
    char* line = nullptr;
    
    while ((line = readline("prompt> ")) != nullptr) {
        if (strlen(line) > 0) {
            add_history(line);  // Add to history
            std::cout << "You entered: " << line << std::endl;
        }
        free(line);
    }
    
    return 0;
}