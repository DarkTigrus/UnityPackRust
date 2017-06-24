/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
#include <stdint.h>
#include <stdio.h>
#include <unitypack.h>

int main() {
    const char* filepath = "/Applications/Hearthstone/Data/OSX/cards0.unity3d";
    uint32_t i;

    struct unitypack_assetbundle* assetbundle = unitypack_load_assetbundle_from_file(filepath);
    printf("Successfully loaded assetbundle from %s\n",filepath);

    uint32_t num_assets = unitypack_get_num_assets(assetbundle);
    printf("There are %d asset(s) in the bundle\n",num_assets);

    for (i=0; i < num_assets; i++) {
        struct unitypack_asset* asset = unitypack_get_asset(assetbundle, i);
        char* asset_name = unitypack_get_asset_name(asset);
        printf("Asset %d: %s\n",i,asset_name);
        unitypack_free_rust_string(asset_name);
    }
    
    unitypack_destroy_assetbundle(assetbundle);
    return 0;
}