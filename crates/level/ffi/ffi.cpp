#include "ffi.h"

#include <iostream>
#include <memory>
#include <cstring>

#include <leveldb/cache.h>
#include <leveldb/db.h>
#include <leveldb/decompress_allocator.h>
#include <leveldb/env.h>
#include <leveldb/filter_policy.h>
#include <leveldb/options.h>
#include <leveldb/status.h>
#include <leveldb/zlib_compressor.h>
#include <leveldb/write_batch.h>

enum FfiStatus translate_status(const leveldb::Status& status) noexcept {
    return static_cast<FfiStatus>(status.code());
}

void copy_string(FfiResult* result, const std::string& error) {
    const char* src = error.c_str();
    size_t src_size = error.size() + 1; // Make space for null.

    result->size = static_cast<int>(src_size);
    result->data = new char[src_size];

    memcpy(result->data, src, src_size);
}

class EmptyLogger : public leveldb::Logger {
public:
    void Logv(const char* fmt, va_list args) override {}
};

struct Database {
    leveldb::Options options = leveldb::Options();
    leveldb::WriteOptions write_options{};
    leveldb::ReadOptions read_options{};
    leveldb::DB* db = nullptr;

    ~Database() noexcept {
        delete this->db;

        delete this->read_options.decompress_allocator;

        delete this->options.compressors[1];
        delete this->options.compressors[0];
        delete this->options.info_log;
        delete this->options.block_cache;
        delete this->options.filter_policy;
    }
};

FfiResult bedrockrs_db_open(const char* path) {
    FfiResult result{};
    try {
        std::unique_ptr<Database> database = std::make_unique<Database>();

        database->options.filter_policy = leveldb::NewBloomFilterPolicy(10);
        database->options.block_cache = leveldb::NewLRUCache(40 * 1024 * 1024);
        database->options.info_log = new EmptyLogger();
        database->options.compressors[0] = new leveldb::ZlibCompressorRaw();
        database->options.compressors[1] = new leveldb::ZlibCompressor();
        
        database->read_options.decompress_allocator = new leveldb::DecompressAllocator();

        leveldb::Status status = leveldb::DB::Open(database->options, path, &database->db);

        result.status = translate_status(status);

        if(status.ok()) {
            result.size = sizeof(Database);
            result.data = database.release();
        } else {    
            std::string error = status.ToString();
            copy_string(&result, error);
        }

        return result;
    } catch(...) {
        result.status = FfiStatus::Exception;
        result.data = nullptr;
        result.size = 0;

        return result;
    }
}

void bedrockrs_db_close(void* db_ptr) {
    try {
        Database* database = reinterpret_cast<Database*>(db_ptr);
        delete database;
    } catch(...) {
        // If we're unable to destroy the database then there's no point in retrying.
        // Just leak it
        std::cout << "exception occurred while freeing database" << std::endl;
    }
}

FfiResult bedrockrs_db_get(void* db_ptr, const char* key, int key_size) {
    FfiResult result{};
    try {
        Database* db = reinterpret_cast<Database*>(db_ptr);

        std::string value;
        leveldb::Status status = db->db->Get(db->read_options, leveldb::Slice(key, key_size), &value);

        result.status = translate_status(status);
        if(status.ok()) {
            result.size = static_cast<int>(value.size());
            result.data = new char[value.size()];

            // TODO: This memcpy can probably be removed.
            memcpy(result.data, value.data(), value.size());
        } else {
            std::string error = status.ToString();
            copy_string(&result, error);
        }

        return result;
    } catch(...) {
        result.status = FfiStatus::Exception;
        result.data = nullptr;
        result.size = 0;

        return result;
    }
}

FfiResult bedrockrs_db_put(
    void* db_ptr,
    const char* key, int key_size,
    const char* val, int val_size
) {
    Database* db = reinterpret_cast<Database*>(db_ptr);
    FfiResult result{};

    try {
        leveldb::Slice key_slice(key, key_size);
        leveldb::Slice val_slice(val, val_size);

        leveldb::Status status = db->db->Put(db->write_options, key_slice, val_slice);
        result.status = translate_status(status);

        if(status.ok()) {
            result.data = nullptr;
            result.size = 0;
        } else {
            std::string error = status.ToString();
            copy_string(&result, error);
        }

        return result;
    } catch(...) {
        result.status = FfiStatus::Exception;
        result.data = nullptr;
        result.size = 0;

        return result;
    }
}

FfiResult bedrockrs_db_remove(void* db_ptr, const char* key, int key_size) {
    Database* db = reinterpret_cast<Database*>(db_ptr);
    FfiResult result{};

    try {
        leveldb::Slice key_slice(key, key_size);

        leveldb::Status status = db->db->Delete(db->write_options, key_slice);

        result.status = translate_status(status);
        if(status.ok()) {
            result.data = nullptr;
            result.size = 0;
        } else {
            std::string error = status.ToString();
            copy_string(&result, error);
        }

        return result;
    } catch(...) {
        result.status = FfiStatus::Exception;
        result.data = nullptr;
        result.size = 0;

        return result;
    }
}

void bedrockrs_buffer_destroy(char* array) {
    try {
        delete[] array;
    } catch(...) {
        // If this fails, there is no point in retrying.
        // Just leak it.
        std::cout << "exception occurred while freeing buffer" << std::endl;
    }
}

FfiResult bedrockrs_iter_new(void* db_ptr) {
    Database* db = reinterpret_cast<Database*>(db_ptr);

    FfiResult result{};    
    try {
        leveldb::Iterator* iter = db->db->NewIterator(db->read_options);
        iter->SeekToFirst();

        result.status = FfiStatus::Success;
        result.size = sizeof(leveldb::Iterator*); // yes this must be the size of the pointer
        result.data = iter;

        return result;
    } catch(...) {
        result.status = FfiStatus::Exception;
        result.data = nullptr;
        result.size = 0;

        return result;
    }
}

void bedrockrs_iter_destroy(void* iter_ptr) {
    try {
        leveldb::Iterator* iter = reinterpret_cast<leveldb::Iterator*>(iter_ptr);
        delete iter;
    } catch(...) {
        // If this fails there is no point in retrying, just leak it.
        std::cout << "exception occurred while freeing iterator" << std::endl;
    }
}

FfiResult bedrockrs_iter_key(const void* iter_ptr) {
    FfiResult result{};
    try {
        const leveldb::Iterator* iter = reinterpret_cast<const leveldb::Iterator*>(iter_ptr);
        leveldb::Slice key = iter->key();

        result.status = FfiStatus::Success;
        result.size = key.size();
        result.data = new char[result.size];
        memcpy(result.data, key.data(), result.size);

        return result;
    } catch(...) {
        result.status = FfiStatus::Exception;
        result.data = nullptr;
        result.size = 0;

        return result;
    }
}

FfiResult bedrockrs_iter_value(const void* iter_ptr) {
    const leveldb::Iterator* iter = reinterpret_cast<const leveldb::Iterator*>(iter_ptr);

    FfiResult result{};
    try {
        leveldb::Slice value = iter->value();

        result.status = FfiStatus::Success;
        result.size = value.size();
        result.data = new char[result.size];
        memcpy(result.data, value.data(), result.size);

        return result;
    } catch(...) {
        result.status = FfiStatus::Exception;
        result.data = nullptr;
        result.size = 0;

        return result;
    }
}

void bedrockrs_iter_next(void* iter_ptr) {
    try {
        leveldb::Iterator* iter = reinterpret_cast<leveldb::Iterator*>(iter_ptr);
        iter->Next();
    } catch(...) {
        // Fail will be caught by `bedrockrs_iter_valid`.
        std::cout << "exception occurred in `Iterator->next`" << std::endl;
    }
}

bool bedrockrs_iter_valid(const void* iter_ptr) {
    const leveldb::Iterator* iter = reinterpret_cast<const leveldb::Iterator*>(iter_ptr);
    try {
        return iter->Valid();
    } catch(...) {
        return false;
    }
}
