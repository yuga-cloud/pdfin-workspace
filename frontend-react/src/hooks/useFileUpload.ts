import { useState, useCallback } from 'react'
import { useMutation } from '@tanstack/react-query'
import { processPdf } from '../lib/api'

export function useFileUpload(operation: string) {
  const [dragActive, setDragActive] = useState(false)
  const [selectedFile, setSelectedFile] = useState<File | null>(null)

  const mutation = useMutation({
    mutationFn: (file: File) => processPdf(file, operation),
  })

  const handleDrag = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true)
    } else if (e.type === 'dragleave') {
      setDragActive(false)
    }
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setDragActive(false)
    const files = e.dataTransfer.files
    if (files?.[0]) {
      handleFile(files[0])
    }
  }, [])

  const handleFile = useCallback((file: File) => {
    setSelectedFile(file)
    mutation.mutate(file)
  }, [mutation])

  return {
    ...mutation,
    dragActive,
    selectedFile,
    handleDrag,
    handleDrop,
    handleFile,
  }
}
