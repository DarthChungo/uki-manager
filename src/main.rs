use exe::SectionCharacteristics;
use exe::PE;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("io operation")]
    Io(#[from] io::Error),

    #[error("exe operation")]
    Exe(#[from] exe::Error),

    #[error("other")]
    Other,
}

struct UnifiedKernelImage {
    executable: exe::VecPE,
    output: String,
}

impl UnifiedKernelImage {
    fn new(stub: &str, output: &str) -> Result<Self, Error> {
        let executable = exe::VecPE::from_disk_file(stub)?;

        Ok(Self {
            executable,
            output: output.to_owned(),
        })
    }

    fn add_section_buf<T: AsRef<[u8]>>(&mut self, name: &str, data: T) -> Result<(), Error> {
        let mut sec = exe::ImageSectionHeader::default();

        let size: u32 = data.as_ref().len().try_into().map_err(|_| Error::Other)?;

        let virtual_size = size;
        let raw_size = self.executable.align_to_file(exe::Offset(size))?.0;

        sec.set_name(Some(name));
        sec.size_of_raw_data = raw_size;
        sec.virtual_size = virtual_size;
        sec.characteristics =
            SectionCharacteristics::MEM_READ | SectionCharacteristics::CNT_INITIALIZED_DATA;

        let sec = self.executable.append_section(&sec)?.to_owned();

        self.executable.resize(
            (sec.pointer_to_raw_data.0 + raw_size)
                .try_into()
                .map_err(|_| Error::Other)?,
            0,
        );

        sec.write(&mut self.executable, data)?;

        Ok(())
    }

    fn add_section_path(&mut self, name: &str, path: &str) -> Result<(), Error> {
        let mut file = fs::File::open(path)?;
        let mut buf: Vec<u8> = Vec::new();

        file.read_to_end(&mut buf)?;

        Ok(self.add_section_buf(name, buf)?)
    }

    fn add_section_paths(&mut self, name: &str, paths: Vec<&str>) -> Result<(), Error> {
        let mut buf: Vec<u8> = Vec::new();

        let files: Vec<fs::File> = paths
            .iter()
            .map(|path| Ok(fs::File::open(path)?))
            .collect::<io::Result<_>>()?;

        files
            .iter()
            .map(|mut f| Ok(f.read_to_end(&mut buf)?))
            .collect::<io::Result<Vec<usize>>>()?;

        Ok(self.add_section_buf(name, buf)?)
    }

    fn output(&mut self) -> Result<(), Error> {
        self.executable.fix_image_size()?;

        let checksum = self.executable.calculate_checksum()?;

        let existing_checksum = match self.executable.get_valid_mut_nt_headers()? {
            exe::NTHeadersMut::NTHeaders32(headers) => &mut headers.optional_header.checksum,
            exe::NTHeadersMut::NTHeaders64(headers) => &mut headers.optional_header.checksum,
        };

        *existing_checksum = checksum;

        let buf = self.executable.recreate_image(exe::PEType::Disk)?;

        let mut output = fs::File::options()
            .write(true)
            .create(true)
            .open(&self.output)?;

        output.write(&buf)?;

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let mut uki =
        UnifiedKernelImage::new("/usr/lib/systemd/boot/efi/linuxx64.efi.stub", "output.efi")?;

    uki.add_section_path(".osrel", "/usr/lib/os-release")?;
    uki.add_section_buf(".uname", "6.4.8-zen1-1-zen")?;
    uki.add_section_path(".cmdline", "/etc/kernel/cmdline")?;
    uki.add_section_paths(
        ".initrd",
        vec!["/boot/intel-ucode.img", "/boot/initramfs-linux-zen.img"],
    )?;
    uki.add_section_path(".linux", "/boot/vmlinuz-linux-zen")?;

    uki.output()?;

    Ok(())
}
