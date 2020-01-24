import os
import shutil
import base64


def get_filenames():
    files = []
    cwd = os.getcwd()
    for filename in os.listdir(cwd):
        if filename.lower().endswith('.autosave') or filename.lower().endswith('.json'):
            files.append(os.path.join(cwd, filename))
    return files


def encode(data, key):
    key_len = len(key)
    return [data[index] ^ key[index % key_len] for index in range(len(data))]

def decode(data, key):
    return encode(data, key)


def make_backup(filepath):
    counter = 1
    new_filepath = "{}.{}".format(filepath, counter)
    while os.path.exists(new_filepath):
        counter += 1
        new_filepath = "{}.{}".format(filepath, counter)
    shutil.copy(filepath, new_filepath)


def is_save_file(filepath):
    return filepath.lower().endswith('.autosave')

def get_save_filename(filepath):
    return filepath.rstrip('.json')

def is_save_file_exists(filepath):
    return os.path.exists(get_save_filename(filepath))


def is_json_file(filepath):
    return filepath.lower().endswith('.json')

def get_json_filename(filepath):
    if filepath.lower().endswith('.json'):
        return filepath
    else:
        return '{}{}'.format(filepath, '.json')

def is_json_exists(filepath):
    return os.path.exists(get_json_filename(filepath))


def decrypt_save_file(filepath):
    with open(filepath, 'rb') as input:
        contents = input.read()

    decoded_once = base64.b64decode(contents)
    decoded_twice = decode(decoded_once, b'key')
    json_text = bytearray(decoded_twice)

    with open(get_json_filename(filepath), 'wb') as output:
        output.write(json_text)


def encrypt_save_file(filepath):
    with open(filepath, 'rb') as input:
        contents = input.read()

    encoded_once_list = encode(contents, b'key')
    encoded_once_bytes = bytearray(encoded_once_list)
    encoded_twice = base64.b64encode(encoded_once_bytes)

    with open(get_save_filename(filepath), 'wb') as output:
        output.write(encoded_twice)


def main():
    for filepath in get_filenames():
        if is_save_file(filepath) and not is_json_exists(filepath):
            print('Decrypting {}...'.format(os.path.basename(filepath)))
            make_backup(filepath)
            decrypt_save_file(filepath)
            continue

        if is_json_file(filepath):
            print('Encrypting {}...'.format(os.path.basename(filepath)))
            encrypt_save_file(filepath)
            os.remove(filepath)
            continue

if __name__ == "__main__":
    main()
