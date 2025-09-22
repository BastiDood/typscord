use std::io;
use tokio::io::{AsyncBufRead, AsyncBufReadExt as _, AsyncRead, AsyncReadExt as _};

pub async fn read_usize<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<usize> {
	let mut bytes = 0usize.to_be_bytes();
	reader.read_exact(&mut bytes).await?;
	Ok(usize::from_be_bytes(bytes))
}

pub async fn read_line<R: AsyncBufRead + Unpin>(
	reader: &mut R,
	buffer: &mut String,
) -> io::Result<String> {
	reader.read_line(buffer).await?;
	let line = buffer.trim().to_owned();
	buffer.clear();
	Ok(line)
}
