#include <SDL2/SDL.h>
#include <iostream>
#include <vector>
#include <string>
#include <thread>
#include <chrono>

int controlInit(); void quitFunc(), handleInput(), consoleFunc(), updateLoop(), helpMenu(), commandFunc();
bool running=true;
SDL_GameController* controller1;
SDL_Event event1;
unsigned int maxValue= 33000;
bool debugOn=false;
int deadzone= maxValue/4, controlValues[]= {0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,0};
int controlIndexes[]={0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23}; //0-3= BRLT button 4-5= LR should 6-8= LRM center 9-10= LR joyButton 11-14= LR XY joystick 15-16= LR trig 17-24= hatDirections
std::string console;
unsigned int updateTimer=15000;
std::string commands[]= {"help", "display", "extended", "updatetimer", "clear", "controlmap", "specialbutton", "debug"};
std::string aliases[]= {"he","ds","ext", "upt", "cls", "ctrlm", "spcb","debug"};

int main(int argc, char* argv[]) {// // /// 
 controlInit();
 helpMenu();
 std::thread t(updateLoop);
 while(running){
  consoleFunc();
 }
 quitFunc();
 return 0;
}

void helpMenu(){
 std::cout<< "commands+ AliAses:\n'HElp' 'DiSplay' 'EXTended'\n'UPdateTimer'("<< updateTimer<< "milliseconds)'\n'CLearS' 'ConTRoLMap'\n'SPeCialButton'\n";
}

void commandFunc(int index, int64_t value=-1){
 if (index==0){ //help
  helpMenu();
 }else if(index==1){ //display
  
 }else if(index==2){ //extended
  
 }else if(index==3){ //updateTimer
  if(value==-1){
   std::cout<<"try 'updatetimer;40000'";
  }else{
   updateTimer= value;
  }
 }else if(index==4){ //clear
  system("clear");
 }else if(index==5){ //controlMap
  
 }else if(index==6){ //specialButton
  
 }else if(index==7){ //debug
  debugOn=!debugOn;
 }
}

void updateLoop(){ 
 if(debugOn){
  std::cout<<"debug";
 }
 while (SDL_PollEvent(&event1)) {
  if (event1.type == SDL_QUIT) {
   running = false;
  }
  handleInput();
 }
 std::this_thread::sleep_for( std::chrono::microseconds( updateTimer));
 updateLoop();
}

void consoleFunc(){
 std::getline(std::cin, console);
 int newLength= std::size(commands);
 for (int i=0; i<newLength; ++i){
  size_t pos= s.find(';');
  if(pos== std::string::npos){
   name=
  }else{

  }
  if (console == commands[i]|| console== aliases[i]) {
   commandFunc(i, );
   break;
  } else if(i==4 && console== "clears"){
   commandFunc(4); break;
  }else if (i== newLength-1){
   std::cout << "unknown: " << console << std::endl;
   break;
  }
 }
}

void handleInput() {
 switch (event1.type) {
  case SDL_JOYBUTTONDOWN:
   controlValues[ controlIndexes[ std::min((int)event1.jbutton.button, 10)]]= maxValue;
   //std::cout << "Button pressed: " <<  << std::endl;
   break;
  case SDL_JOYAXISMOTION: {
   int newAxis= (int)event1.jaxis.axis;
   if (abs(event1.jaxis.value) > 8000) {
    controlValues[ controlIndexes[ std::min(newAxis- newAxis/2, 4)+ 11]]= maxValue;
    //std::cout << "Axis " << (int)event1.jaxis.axis << ": " << event1.jaxis.value << std::endl;
   }
   break;
  }
  case SDL_JOYHATMOTION:
   int hatI= (int)event1.jhat.hat;
   if (hatI==0){
    for(int i=0; i<8; i++){
     controlValues[ controlIndexes[ i+ 17]]= 0;
    }
   }else{
    controlValues[ controlIndexes[ (int)event1.jhat.hat+ 17]]= maxValue;
   }
   //std::cout << "Hat " << (int)event1.jhat.hat << ": " << (int)event1.jhat.value << std::endl;
   break;
 }
}

int controlInit(){
 if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_JOYSTICK) != 0) {
  std::cerr << "SDL_Init error: " << SDL_GetError() << std::endl;
  return 1;
 }

 int numJoysticks = SDL_NumJoysticks();
 if (numJoysticks == 0) {
  std::cerr << "No joysticks found!" << std::endl;
  SDL_Quit();
  return 1;
 }

 controller1 = SDL_GameControllerOpen(0);
 if (!controller1) {
  std::cerr << "Could not open game controller: " << SDL_GetError() << std::endl;
  SDL_Quit();
  return 1;
 }
 return 0;
}

void quitFunc(){
 SDL_GameControllerClose(controller1);
 SDL_Quit();
}
