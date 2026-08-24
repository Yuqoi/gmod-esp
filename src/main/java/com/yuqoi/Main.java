package com.yuqoi;

//use g_pEngineClient->WorldToScreenMatrix(), g_pEngineClient->GetMaxClients(), get all other players, check to see if player is valid (aka alive and some other checks), get player origins with GetAbsOrigin, hook PaintTraverse and draw to screen. you have to get these values at certain frames. (hint hint) when everything is done your esp will flicker if you're on x64. you can fix this by double buffering

public class Main {
    static void main() {

        int a = 15;

        System.out.println(System.identityHashCode(a));

    }
}
