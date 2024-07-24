package athcon

/*
#include "../../athcon.h"
*/
import "C"

type CallKind int

const (
	Call CallKind = C.ATHCON_CALL
)

type StorageStatus int

const (
	StorageAssigned         StorageStatus = C.ATHCON_STORAGE_ASSIGNED
	StorageAdded            StorageStatus = C.ATHCON_STORAGE_ADDED
	StorageDeleted          StorageStatus = C.ATHCON_STORAGE_DELETED
	StorageModified         StorageStatus = C.ATHCON_STORAGE_MODIFIED
	StorageDeletedAdded     StorageStatus = C.ATHCON_STORAGE_DELETED_ADDED
	StorageModifiedDeleted  StorageStatus = C.ATHCON_STORAGE_MODIFIED_DELETED
	StorageDeletedRestored  StorageStatus = C.ATHCON_STORAGE_DELETED_RESTORED
	StorageAddedDeleted     StorageStatus = C.ATHCON_STORAGE_ADDED_DELETED
	StorageModifiedRestored StorageStatus = C.ATHCON_STORAGE_MODIFIED_RESTORED
)
