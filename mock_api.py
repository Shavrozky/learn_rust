from typing import Annotated
import asyncio
from fastapi import FastAPI, File, UploadFile
import uvicorn

app = FastAPI()

@app.post("/process-image")
async def process_image(file: Annotated[UploadFile, File()]):
    print(f"Menerima gambar: {file.filename} dari backend Rust!")
    
    image_bytes = await file.read()
    print(f"Ukuran file: {len(image_bytes)} bytes. Memproses OCR...")
    
    await asyncio.sleep(2)
    
    extracted_text = f"BRG-OCR-{file.filename.split('.')[0].upper()}"
    print(f"Hasil ekstraksi: {extracted_text}")
    
    return {"extracted_text": extracted_text}

if __name__ == "__main__":
    uvicorn.run("mock_api:app", host="127.0.0.1", port=8000, reload=True)