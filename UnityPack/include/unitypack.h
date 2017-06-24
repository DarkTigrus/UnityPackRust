/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
#ifndef UNITYPACK_H
#define UNITYPACK_H

/* Asset bundle structure */
struct unitypack_assetbundle;
struct unitypack_asset;

/* Loads and returns an assetbundle from the give path. Unity assetbundles are usually with .unity3d extension. */
extern const struct unitypack_assetbundle* unitypack_load_assetbundle_from_file(const char* filepath);

/* Destroys a previously loaded assetbundle */
extern void unitypack_destroy_assetbundle(const struct unitypack_assetbundle*);

/* Returns the number of assets inside the given bundle */
extern uint32_t unitypack_get_num_assets(const struct unitypack_assetbundle*);

/* Returns the asset at the given index */
extern struct unitypack_asset* unitypack_get_asset(const struct unitypack_assetbundle*, uint32_t i);

/* Returns the name of the asset */
extern const char* unitypack_get_asset_name(struct unitypack_asset*);

/* Frees a c string created by the unitypack library */
extern void unitypack_free_rust_string(const char*);

extern uint32_t unitypack_get_num_objects(struct unitypack_asset*);

#endif /* UNITYPACK_H */
